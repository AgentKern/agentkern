use agentkern_gate::loader::{FilePolicyLoader, PolicyLoader};
use std::path::PathBuf;
use tokio::time::{sleep, Duration};

#[tokio::test]
async fn test_policy_loading_from_file() {
    // Point to the real policies directory relative to this test file
    // tests/ is in packages/pillars/gate/tests
    // policies/ is in root policies/
    let mut policy_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    policy_dir.pop(); // pillars/gate
    policy_dir.pop(); // pillars
    policy_dir.pop(); // packages
    policy_dir.push("policies");

    // Ensure it exists
    assert!(policy_dir.exists(), "Policies directory not found at {:?}", policy_dir);

    let loader = FilePolicyLoader::new(policy_dir);
    let policies = loader.load_all().await.expect("Failed to load policies");

    assert!(!policies.is_empty(), "Should load at least one policy");
    
    // Check for core system guardrails
    let core_policy = policies.iter().find(|p| p.id == "core-system-guardrails");
    assert!(core_policy.is_some(), "Core system guardrails policy not found");

    let p = core_policy.unwrap();
    assert_eq!(p.priority, 1000);
    assert!(p.rules.iter().any(|r| r.id == "prevent-db-deletion"));
}
