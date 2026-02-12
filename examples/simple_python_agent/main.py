import os
import time
from agentkern import Agent

def main():
    print("🤖 AgentKern: Simple Python Agent Reference")
    print("------------------------------------------")

    # 1. Initialize Agent Identity (Managed in Rust, used in Python)
    # This generates an Ed25519 keypair and a DID (Decentralized Identifier)
    agent_name = "example-trader"
    trader = Agent.generate(agent_name)
    print(f"✅ Identity Verified: {trader.name}")
    print(f"🆔 DID: {trader.id}")

    # 2. Simulate an Agent Loop
    actions = [
        {"type": "market_scan", "params": {"symbol": "BTC/USD"}},
        {"type": "execute_trade", "params": {"amount": 50, "side": "buy"}},
        {"type": "execute_trade", "params": {"amount": 5000, "side": "buy"}}, # This should trigger a policy block
    ]

    for action_req in actions:
        action_type = action_req["type"]
        params = action_req["params"]
        
        print(f"\nEvaluating action: {action_type} ({params})")

        # 3. Policy Verification (The 'Gate' Pillar)
        # In a real setup, this would hit a local or remote Gate Server
        # For this example, we simulate the logic that the Gate Server would perform
        
        allowed = True
        reason = "Passed local checks"

        if action_type == "execute_trade" and params["amount"] > 1000:
            allowed = False
            reason = "Amount exceeds $1,000 safety threshold"

        if allowed:
            # 4. Generate Liability Proof
            # All authorized actions should be signed for non-repudiation
            proof = trader.create_proof(action_type)
            print(f"🟢 AUTHORIZED: {reason}")
            print(f"📄 Proof generated: {proof.jti}")
            
            # Execute the actual logic here (e.g., calling an exchange API)
            print(f"🚀 Executing {action_type}...")
        else:
            print(f"🔴 BLOCKED: {reason}")

    print("\n------------------------------------------")
    print("🎉 Integration demo complete.")

if __name__ == "__main__":
    main()
