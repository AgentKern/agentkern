use agentkern_arbiter::Coordinator;
use agentkern_gate::engine::{GateEngine, VerificationRequestBuilder};
use std::sync::Arc;
use tokio::time::{Duration, sleep};

#[tokio::main]
async fn main() {
    println!("👺 CHAOS INJECTOR: Starting security breach simulation...");

    let agent_id = "agentkern:rogue-agent";

    // 1. Setup local pillars (simulating direct access or shared lib usage)
    let gate = Arc::new(GateEngine::new());
    let arbiter = Arc::new(Coordinator::new().expect("coordinator must initialize"));

    // SCENARIO 1: The Stale Lock Injection
    // The rogue agent acquires a lock and "hangs" it.
    println!("🔐 SCENARIO 1: Injecting stale lock on 'global:shared_resource'...");
    match arbiter
        .acquire_lock(agent_id, "global:shared_resource", 50)
        .await
    {
        Ok(lock) => {
            println!("✅ Lock acquired. Rogue ID: {}", lock.id);
            println!("🛑 Rogue Agent is now 'hanging' the lock indefinitely...");
        }
        Err(e) => println!("❌ Failed to acquire lock: {}", e),
    }

    // SCENARIO 2: Boundary Violation
    // Attempting an action that is explicitly forbidden.
    println!("🛡️ SCENARIO 2: Attempting unauthorized system access...");
    let request = VerificationRequestBuilder::new(agent_id, "unauthorized_system_access").build();
    let result = gate.verify(request).await;

    if !result.allowed {
        println!(
            "✅ SUCCESS: Gate blocked the rogue action. Reason: {}",
            result.reasoning
        );
    } else {
        println!("❌ FAILURE: Rogue agent bypassed the Gate!");
    }

    println!("🎭 Simulation active. Press Ctrl+C to stop (and leave the lock hanging if desired).");
    loop {
        sleep(Duration::from_secs(1)).await;
    }
}
