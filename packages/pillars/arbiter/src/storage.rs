use std::fmt::Debug;
use std::path::Path;
use std::sync::Arc;

use ::openraft::storage::{
    LogFlushed, LogState, RaftLogReader, RaftLogStorage, RaftSnapshotBuilder, RaftStateMachine,
    Snapshot, SnapshotMeta,
};
use ::openraft::{
    AnyError, EntryPayload, LogId, OptionalSend, RaftTypeConfig, StorageError, StorageIOError,
    StoredMembership, Vote,
};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

use crate::raft_manager::{LockCommand, LockStateMachine};

/// Raft Type Configuration
#[derive(
    Debug, Clone, Copy, Default, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize,
)]
pub struct TypeConfig {}

impl RaftTypeConfig for TypeConfig {
    type D = LockCommand;
    type R = bool;
    type NodeId = u64;
    type Node = ();
    type Entry = ::openraft::Entry<Self>;
    type SnapshotData = std::io::Cursor<Vec<u8>>;
    type AsyncRuntime = ::openraft::TokioRuntime;
    type Responder = ::openraft::impls::OneshotResponder<Self>;
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LockSnapshot {
    pub logs: Vec<u8>,
    pub metadata: Vec<u8>,
}

#[derive(Clone)]
pub struct SledStore {
    db: sled::Db,
    logs: sled::Tree,
    store: sled::Tree,
    state_machine: Arc<RwLock<LockStateMachine>>,
}

impl SledStore {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        let db = sled::open(path).expect("Failed to open sled db");
        let logs = db.open_tree("logs").expect("Failed to open logs tree");
        let store = db.open_tree("store").expect("Failed to open store tree");

        // Load state machine if it exists
        let sm = if let Ok(Some(v)) = store.get(b"state_machine") {
            serde_json::from_slice(&v).unwrap_or_else(|_| LockStateMachine::new())
        } else {
            LockStateMachine::new()
        };

        Self {
            db,
            logs,
            store,
            state_machine: Arc::new(RwLock::new(sm)),
        }
    }

    fn get_last_log_id(&self) -> Option<LogId<u64>> {
        match self.logs.last() {
            Ok(Some((_key, val))) => {
                let entry: ::openraft::Entry<TypeConfig> = serde_json::from_slice(&val).ok()?;
                Some(entry.log_id)
            }
            _ => None,
        }
    }
}

impl RaftLogReader<TypeConfig> for SledStore {
    async fn try_get_log_entries<RB: std::ops::RangeBounds<u64> + Clone + Debug + OptionalSend>(
        &mut self,
        range: RB,
    ) -> Result<Vec<::openraft::Entry<TypeConfig>>, StorageError<u64>> {
        let start = match range.start_bound() {
            std::ops::Bound::Included(i) => *i,
            std::ops::Bound::Excluded(i) => *i + 1,
            std::ops::Bound::Unbounded => 0,
        };

        let mut entries = Vec::new();
        for item in self.logs.range(start.to_be_bytes()..) {
            let (key, val) = item.map_err(|e| StorageIOError::read_logs(AnyError::new(&e)))?;
            let idx = u64::from_be_bytes(key[..8].try_into().unwrap());
            if let std::ops::Bound::Excluded(e) = range.end_bound() {
                if idx >= *e {
                    break;
                }
            }
            if let std::ops::Bound::Included(e) = range.end_bound() {
                if idx > *e {
                    break;
                }
            }
            let entry: ::openraft::Entry<TypeConfig> = serde_json::from_slice(&val)
                .map_err(|e| StorageIOError::read_logs(AnyError::new(&e)))?;
            entries.push(entry);
        }
        Ok(entries)
    }
}

impl RaftLogStorage<TypeConfig> for SledStore {
    type LogReader = Self;

    async fn get_log_state(&mut self) -> Result<LogState<TypeConfig>, StorageError<u64>> {
        let last = self.get_last_log_id();
        let last_purged = self
            .store
            .get(b"last_purged_log_id")
            .map_err(|e| StorageIOError::read_logs(AnyError::new(&e)))?;

        let last_purged_log_id = if let Some(v) = last_purged {
            serde_json::from_slice(&v).ok()
        } else {
            None
        };

        Ok(LogState {
            last_purged_log_id,
            last_log_id: last,
        })
    }

    async fn get_log_reader(&mut self) -> Self::LogReader {
        self.clone()
    }

    async fn save_vote(&mut self, vote: &Vote<u64>) -> Result<(), StorageError<u64>> {
        let val =
            serde_json::to_vec(vote).map_err(|e| StorageIOError::write_vote(AnyError::new(&e)))?;
        self.store
            .insert(b"vote", val)
            .map_err(|e| StorageIOError::write_vote(AnyError::new(&e)))?;
        self.db
            .flush()
            .map_err(|e| StorageIOError::write_vote(AnyError::new(&e)))?;
        Ok(())
    }

    async fn read_vote(&mut self) -> Result<Option<Vote<u64>>, StorageError<u64>> {
        let val = self
            .store
            .get(b"vote")
            .map_err(|e| StorageIOError::read_vote(AnyError::new(&e)))?;
        if let Some(v) = val {
            let vote = serde_json::from_slice(&v)
                .map_err(|e| StorageIOError::read_vote(AnyError::new(&e)))?;
            Ok(Some(vote))
        } else {
            Ok(None)
        }
    }

    async fn append<I>(
        &mut self,
        entries: I,
        callback: LogFlushed<TypeConfig>,
    ) -> Result<(), StorageError<u64>>
    where
        I: IntoIterator<Item = ::openraft::Entry<TypeConfig>> + OptionalSend,
        I::IntoIter: OptionalSend,
    {
        for entry in entries {
            let id_bytes = entry.log_id.index.to_be_bytes();
            let val = serde_json::to_vec(&entry)
                .map_err(|e| StorageIOError::write_logs(AnyError::new(&e)))?;
            self.logs
                .insert(id_bytes, val)
                .map_err(|e| StorageIOError::write_logs(AnyError::new(&e)))?;
        }
        self.db
            .flush()
            .map_err(|e| StorageIOError::write_logs(AnyError::new(&e)))?;
        callback.log_io_completed(Ok(()));
        Ok(())
    }

    async fn truncate(&mut self, log_id: LogId<u64>) -> Result<(), StorageError<u64>> {
        let start = log_id.index.to_be_bytes();
        let keys: Vec<_> = self.logs.range(start..).keys().collect();
        for key in keys {
            let k = key.map_err(|e| StorageIOError::write_logs(AnyError::new(&e)))?;
            self.logs
                .remove(k)
                .map_err(|e| StorageIOError::write_logs(AnyError::new(&e)))?;
        }
        self.db
            .flush()
            .map_err(|e| StorageIOError::write_logs(AnyError::new(&e)))?;
        Ok(())
    }

    async fn purge(&mut self, log_id: LogId<u64>) -> Result<(), StorageError<u64>> {
        let keys: Vec<_> = self.logs.iter().keys().collect();
        for key in keys {
            let k = key.map_err(|e| StorageIOError::write_logs(AnyError::new(&e)))?;
            let idx = u64::from_be_bytes(k[..8].try_into().unwrap());
            if idx <= log_id.index {
                self.logs
                    .remove(k)
                    .map_err(|e| StorageIOError::write_logs(AnyError::new(&e)))?;
            } else {
                break;
            }
        }
        let val = serde_json::to_vec(&log_id)
            .map_err(|e| StorageIOError::write_logs(AnyError::new(&e)))?;
        self.store
            .insert(b"last_purged_log_id", val)
            .map_err(|e| StorageIOError::write_logs(AnyError::new(&e)))?;
        self.db
            .flush()
            .map_err(|e| StorageIOError::write_logs(AnyError::new(&e)))?;
        Ok(())
    }
}

impl RaftSnapshotBuilder<TypeConfig> for SledStore {
    async fn build_snapshot(&mut self) -> Result<Snapshot<TypeConfig>, StorageError<u64>> {
        let (last_applied, membership) = self.applied_state().await?;

        let last_applied = last_applied.ok_or_else(|| {
            StorageError::from(StorageIOError::read(AnyError::error(
                "No applied state for snapshot",
            )))
        })?;

        let sm = self.state_machine.read();
        let data = serde_json::to_vec(&*sm).map_err(|e| StorageIOError::read(AnyError::new(&e)))?;

        let snapshot_id = format!("{}-{}", last_applied.leader_id, last_applied.index);

        let meta = SnapshotMeta {
            last_log_id: Some(last_applied),
            last_membership: membership,
            snapshot_id,
        };

        Ok(Snapshot {
            meta,
            snapshot: Box::new(std::io::Cursor::new(data)),
        })
    }
}

impl RaftStateMachine<TypeConfig> for SledStore {
    type SnapshotBuilder = Self;

    async fn applied_state(
        &mut self,
    ) -> Result<(Option<LogId<u64>>, StoredMembership<u64, ()>), StorageError<u64>> {
        let val = self
            .store
            .get(b"last_applied")
            .map_err(|e| StorageIOError::read(AnyError::new(&e)))?;
        let log_id = if let Some(v) = val {
            serde_json::from_slice(&v).ok()
        } else {
            None
        };

        let mem_val = self
            .store
            .get(b"membership")
            .map_err(|e| StorageIOError::read(AnyError::new(&e)))?;
        let membership = if let Some(v) = mem_val {
            serde_json::from_slice(&v).unwrap_or_default()
        } else {
            StoredMembership::default()
        };

        Ok((log_id, membership))
    }

    async fn apply<I>(&mut self, entries: I) -> Result<Vec<bool>, StorageError<u64>>
    where
        I: IntoIterator<Item = ::openraft::Entry<TypeConfig>> + OptionalSend,
        I::IntoIter: OptionalSend,
    {
        let entries_vec: Vec<_> = entries.into_iter().collect();
        let mut res = Vec::with_capacity(entries_vec.len());
        let mut sm = self.state_machine.write();

        for entry in entries_vec {
            match entry.payload {
                EntryPayload::Normal(ref cmd) => match sm.apply(cmd) {
                    Ok(r) => res.push(r),
                    Err(_) => res.push(false),
                },
                EntryPayload::Membership(ref mem) => {
                    let membership = StoredMembership::new(Some(entry.log_id), mem.clone());
                    let val = serde_json::to_vec(&membership)
                        .map_err(|e| StorageIOError::write_logs(AnyError::new(&e)))?;
                    self.store
                        .insert(b"membership", val)
                        .map_err(|e| StorageIOError::write_logs(AnyError::new(&e)))?;
                    res.push(true);
                }
                _ => {
                    res.push(false);
                }
            }

            let val = serde_json::to_vec(&entry.log_id)
                .map_err(|e| StorageIOError::write_logs(AnyError::new(&e)))?;
            self.store
                .insert(b"last_applied", val)
                .map_err(|e| StorageIOError::write_logs(AnyError::new(&e)))?;
        }

        // Persist state machine after applying all entries
        let sm_data =
            serde_json::to_vec(&*sm).map_err(|e| StorageIOError::write_logs(AnyError::new(&e)))?;
        self.store
            .insert(b"state_machine", sm_data)
            .map_err(|e| StorageIOError::write_logs(AnyError::new(&e)))?;

        self.db
            .flush()
            .map_err(|e| StorageIOError::write_logs(AnyError::new(&e)))?;
        Ok(res)
    }

    async fn get_snapshot_builder(&mut self) -> Self::SnapshotBuilder {
        self.clone()
    }

    async fn begin_receiving_snapshot(
        &mut self,
    ) -> Result<Box<std::io::Cursor<Vec<u8>>>, StorageError<u64>> {
        Ok(Box::new(std::io::Cursor::new(Vec::new())))
    }

    async fn install_snapshot(
        &mut self,
        meta: &SnapshotMeta<u64, ()>,
        snapshot: Box<std::io::Cursor<Vec<u8>>>,
    ) -> Result<(), StorageError<u64>> {
        let new_sm: LockStateMachine = serde_json::from_slice(snapshot.get_ref())
            .map_err(|e| StorageIOError::read(AnyError::new(&e)))?;

        {
            let mut sm = self.state_machine.write();
            *sm = new_sm;

            // Persist the new state machine
            let sm_data = serde_json::to_vec(&*sm)
                .map_err(|e| StorageIOError::write_logs(AnyError::new(&e)))?;
            self.store
                .insert(b"state_machine", sm_data)
                .map_err(|e| StorageIOError::write_logs(AnyError::new(&e)))?;

            // Persist the new membership
            let mem_data = serde_json::to_vec(&meta.last_membership)
                .map_err(|e| StorageIOError::write_logs(AnyError::new(&e)))?;
            self.store
                .insert(b"membership", mem_data)
                .map_err(|e| StorageIOError::write_logs(AnyError::new(&e)))?;
        }

        let val = serde_json::to_vec(&meta.last_log_id)
            .map_err(|e| StorageIOError::write_logs(AnyError::new(&e)))?;
        self.store
            .insert(b"last_applied", val)
            .map_err(|e| StorageIOError::write_logs(AnyError::new(&e)))?;
        self.db
            .flush()
            .map_err(|e| StorageIOError::write_logs(AnyError::new(&e)))?;
        Ok(())
    }

    async fn get_current_snapshot(
        &mut self,
    ) -> Result<Option<Snapshot<TypeConfig>>, StorageError<u64>> {
        let (last_applied, membership) = self.applied_state().await?;
        let last_applied = match last_applied {
            Some(id) => id,
            None => return Ok(None),
        };

        let sm = self.state_machine.read();
        let data = serde_json::to_vec(&*sm).map_err(|e| StorageIOError::read(AnyError::new(&e)))?;

        let snapshot_id = format!("{}-{}", last_applied.leader_id, last_applied.index);

        Ok(Some(Snapshot {
            meta: SnapshotMeta {
                last_log_id: Some(last_applied),
                last_membership: membership,
                snapshot_id,
            },
            snapshot: Box::new(std::io::Cursor::new(data)),
        }))
    }
}
