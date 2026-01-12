//! Kani Formal Verification for Treasury
//!
//! This module contains proofs that the Treasury's 2-phase commit
//! maintains balance conservation invariant.

#[cfg(kani)]
mod proofs {
    use crate::balance::{BalanceLedger, Currency};
    use crate::transfer::{TransferEngine, TransferRequest};
    use crate::types::Amount;
    use std::sync::Arc;

    #[kani::proof]
    #[kani::unwind(2)]
    fn verify_transfer_atomicity() {
        let ledger = Arc::new(BalanceLedger::new(Currency::VMC));
        
        // Symbolic starting balances
        let balance_a: u64 = kani::any();
        let balance_b: u64 = kani::any();
        let transfer_amount: u64 = kani::any();
        
        // Constraints to avoid overflow in setup
        kani::assume(balance_a < 1_000_000);
        kani::assume(balance_b < 1_000_000);
        kani::assume(transfer_amount < 1_000_000);
        
        ledger.deposit("agent-a", Amount::from_raw(balance_a)).unwrap();
        ledger.deposit("agent-b", Amount::from_raw(balance_b)).unwrap();
        
        let initial_total = balance_a + balance_b;
        
        let engine = TransferEngine::new(ledger.clone());
        let rt = tokio::runtime::Runtime::new().unwrap();
        
        rt.block_on(async {
            let request = TransferRequest::new("agent-a", "agent-b", Amount::from_raw(transfer_amount));
            let result = engine.transfer(request).await;
            
            let final_a = ledger.get_balance("agent-a").unwrap().as_raw();
            let final_b = ledger.get_balance("agent-b").unwrap().as_raw();
            let final_total = final_a + final_b;
            
            // Invariant: Total balance must be conserved
            // Note: If transfer failed, total remains same. If succeeded, total remains same.
            kani::assert(initial_total == final_total, "Balance conservation failed");
            
            if result.status == crate::transfer::TransferStatus::Completed {
                kani::assert(final_a == balance_a - transfer_amount, "Sender balance mismatch");
                kani::assert(final_b == balance_b + transfer_amount, "Receiver balance mismatch");
            } else {
                kani::assert(final_a == balance_a, "Failed transfer altered sender balance");
                kani::assert(final_b == balance_b, "Failed transfer altered receiver balance");
            }
        });
    }
}
