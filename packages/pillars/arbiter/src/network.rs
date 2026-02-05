use std::collections::HashMap;
use std::sync::Arc;

use ::openraft::error::{NetworkError, RPCError, RaftError, InstallSnapshotError, RemoteError};
use ::openraft::raft::{AppendEntriesRequest, AppendEntriesResponse, InstallSnapshotRequest, InstallSnapshotResponse, VoteRequest, VoteResponse};
use ::openraft::{RaftNetwork, RaftNetworkFactory};
use parking_lot::RwLock;

use crate::storage::TypeConfig;

#[derive(Clone)]
pub struct Network {
    /// Mapping from node id to its address (e.g., "127.0.0.1:3001")
    nodes: Arc<RwLock<HashMap<u64, String>>>,
    client: reqwest::Client,
}

impl Network {
    pub fn new() -> Self {
        Self {
            nodes: Arc::new(RwLock::new(HashMap::new())),
            client: reqwest::Client::new(),
        }
    }

    pub fn register_node(&self, node_id: u64, addr: String) {
        self.nodes.write().insert(node_id, addr);
    }
}

impl RaftNetworkFactory<TypeConfig> for Network {
    type Network = NetworkConnection;

    async fn new_client(&mut self, target: u64, _node: &()) -> Self::Network {
        let addr = self.nodes.read().get(&target).cloned().unwrap_or_else(|| {
            tracing::warn!("Target node {} not found in registry", target);
            format!("127.0.0.1:{}", 3000 + target) // Fallback heuristic
        });

        NetworkConnection {
            addr,
            client: self.client.clone(),
        }
    }
}

pub struct NetworkConnection {
    addr: String,
    client: reqwest::Client,
}

impl RaftNetwork<TypeConfig> for NetworkConnection {
    async fn append_entries(
        &mut self,
        rpc: AppendEntriesRequest<TypeConfig>,
        _option: ::openraft::network::RPCOption,
    ) -> Result<AppendEntriesResponse<u64>, RPCError<u64, (), RaftError<u64>>> {
        let url = format!("http://{}/raft/append", self.addr);
        let resp = self.client.post(&url)
            .json(&rpc)
            .send()
            .await
            .map_err(|e| RPCError::Network(NetworkError::new(&e)))?;

        let res: Result<AppendEntriesResponse<u64>, RaftError<u64>> = resp.json()
            .await
            .map_err(|e| RPCError::Network(NetworkError::new(&e)))?;

        res.map_err(|e| RPCError::RemoteError(RemoteError::new(0, e))) // NodeId placeholder
    }

    async fn install_snapshot(
        &mut self,
        rpc: InstallSnapshotRequest<TypeConfig>,
        _option: ::openraft::network::RPCOption,
    ) -> Result<InstallSnapshotResponse<u64>, RPCError<u64, (), RaftError<u64, InstallSnapshotError>>> {
        let url = format!("http://{}/raft/snapshot", self.addr);
        let resp = self.client.post(&url)
            .json(&rpc)
            .send()
            .await
            .map_err(|e| RPCError::Network(NetworkError::new(&e)))?;

        let res: Result<InstallSnapshotResponse<u64>, RaftError<u64, InstallSnapshotError>> = resp.json()
            .await
            .map_err(|e| RPCError::Network(NetworkError::new(&e)))?;

        res.map_err(|e| RPCError::RemoteError(RemoteError::new(0, e)))
    }

    async fn vote(
        &mut self,
        rpc: VoteRequest<u64>,
        _option: ::openraft::network::RPCOption,
    ) -> Result<VoteResponse<u64>, RPCError<u64, (), RaftError<u64>>> {
        let url = format!("http://{}/raft/vote", self.addr);
        let resp = self.client.post(&url)
            .json(&rpc)
            .send()
            .await
            .map_err(|e| RPCError::Network(NetworkError::new(&e)))?;

        let res: Result<VoteResponse<u64>, RaftError<u64>> = resp.json()
            .await
            .map_err(|e| RPCError::Network(NetworkError::new(&e)))?;

        res.map_err(|e| RPCError::RemoteError(RemoteError::new(0, e)))
    }
}
