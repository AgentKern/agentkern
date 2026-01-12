use agentkern_arbiter::{ConsensusVote, CoordinationRequest, Coordinator, LockType, ProposalType};
use rust_decimal::Decimal;

#[tokio::test]
async fn test_consensus_security_override() {
    let coordinator = Coordinator::new();
    let consensus = coordinator.consensus();

    let resource = "database:restricted";

    // 1. Initial request should be blocked by symbolic rule (mock intent)
    let blocked_req = CoordinationRequest::new("agent-1", resource)
        .with_operation(LockType::Exclusive)
        .with_intent("delete from database:restricted");

    let result = coordinator.request(blocked_req).await;
    assert!(!result.granted);
    assert!(result.reason.unwrap().contains("Sovereign Security"));

    // 2. Propose a security override
    let proposal_id = consensus
        .propose(
            "admin-agent",
            ProposalType::SecurityOverride {
                resource: resource.to_string(),
                reason: "Emergency maintenance".to_string(),
            },
            2, // Threshold of 2 votes
        )
        .await;

    // 3. Cast votes
    consensus
        .vote(proposal_id, "agent-2", ConsensusVote::Aye)
        .await;
    consensus
        .vote(proposal_id, "agent-3", ConsensusVote::Aye)
        .await;

    // 4. Verification: The same request should now pass due to override
    let override_req = CoordinationRequest::new("agent-1", resource)
        .with_operation(LockType::Exclusive)
        .with_intent("delete from database:restricted");

    let result = coordinator.request(override_req).await;
    assert!(
        result.granted,
        "Request should be granted after consensus override"
    );
}

#[tokio::test]
async fn test_consensus_budget_override() {
    let coordinator = Coordinator::new();
    let consensus = coordinator.consensus();

    let agent_id = "agent-rich";

    // 1. Propose budget increase
    let proposal_id = consensus
        .propose(
            "manager-agent",
            ProposalType::BudgetIncrease {
                agent_id: agent_id.to_string(),
                amount_usd: Decimal::from(50),
            },
            1,
        )
        .await;

    consensus
        .vote(proposal_id, "agent-boss", ConsensusVote::Aye)
        .await;

    // 2. Verify budget override in consensus engine
    let budget = consensus.get_budget_override(agent_id).await;
    assert_eq!(budget, Decimal::from(50));

    // 3. Coordinator request should now respect $10 + $50 = $60 budget
    // Verification logic here...
}
