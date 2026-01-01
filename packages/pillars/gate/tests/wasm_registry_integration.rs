//! Integration Tests for WASM Registry
//!
//! Tests hot-swap, capability routing, and module lifecycle.

#[cfg(feature = "wasm")]
mod wasm_registry_tests {
    use agentkern_gate::wasm::{Capability, WasmRegistry};

    fn sample_wasm_module() -> Vec<u8> {
        // Minimal valid WASM module (WAT -> binary)
        // (module (func (export "evaluate")))
        wat::parse_str(r#"(module (func (export "evaluate")))"#).expect("Failed to parse WAT")
    }

    #[tokio::test]
    async fn test_register_and_list() {
        let registry = WasmRegistry::new().expect("Registry creation failed");

        let caps = vec![Capability {
            name: "test_cap".to_string(),
            input_schema: None,
            output_schema: None,
        }];

        let meta = registry
            .register("test-actor", "1.0.0", &sample_wasm_module(), caps)
            .expect("Registration failed");

        assert_eq!(meta.name, "test-actor");
        assert_eq!(meta.version, "1.0.0");

        let actors = registry.list_actors();
        assert_eq!(actors.len(), 1);
    }

    #[tokio::test]
    async fn test_hot_swap() {
        let registry = WasmRegistry::new().expect("Registry creation failed");

        let caps = vec![Capability {
            name: "prompt_guard".to_string(),
            input_schema: None,
            output_schema: None,
        }];

        // Register v1
        registry
            .register("policy", "1.0.0", &sample_wasm_module(), caps.clone())
            .expect("v1 registration failed");

        let v1 = registry.get_actor("policy").unwrap();
        assert_eq!(v1.version, "1.0.0");

        // Hot-swap to v2
        registry
            .register("policy", "2.0.0", &sample_wasm_module(), caps)
            .expect("v2 registration failed");

        let v2 = registry.get_actor("policy").unwrap();
        assert_eq!(v2.version, "2.0.0");

        // Still only 1 actor
        assert_eq!(registry.list_actors().len(), 1);
    }

    #[tokio::test]
    async fn test_capability_routing() {
        let registry = WasmRegistry::new().expect("Registry creation failed");

        // Actor 1: prompt_guard capability
        registry
            .register(
                "actor1",
                "1.0.0",
                &sample_wasm_module(),
                vec![Capability {
                    name: "prompt_guard".to_string(),
                    input_schema: None,
                    output_schema: None,
                }],
            )
            .unwrap();

        // Actor 2: carbon_check capability
        registry
            .register(
                "actor2",
                "1.0.0",
                &sample_wasm_module(),
                vec![Capability {
                    name: "carbon_check".to_string(),
                    input_schema: None,
                    output_schema: None,
                }],
            )
            .unwrap();

        // Route by capability
        let prompt_actors = registry.route_by_capability("prompt_guard");
        assert!(prompt_actors.contains(&"actor1".to_string()));

        let carbon_actors = registry.route_by_capability("carbon_check");
        assert!(carbon_actors.contains(&"actor2".to_string()));

        // Unknown capability
        let unknown = registry.route_by_capability("unknown");
        assert!(unknown.is_empty());
    }

    #[tokio::test]
    async fn test_unregister() {
        let registry = WasmRegistry::new().expect("Registry creation failed");

        registry
            .register("temp", "1.0.0", &sample_wasm_module(), vec![])
            .unwrap();
        assert_eq!(registry.list_actors().len(), 1);

        let removed = registry.unregister("temp");
        assert!(removed);
        assert_eq!(registry.list_actors().len(), 0);

        // Double unregister returns false
        assert!(!registry.unregister("temp"));
    }

    #[tokio::test]
    async fn test_stats() {
        let registry = WasmRegistry::new().expect("Registry creation failed");

        registry
            .register("a1", "1.0.0", &sample_wasm_module(), vec![])
            .unwrap();
        registry
            .register("a2", "1.0.0", &sample_wasm_module(), vec![])
            .unwrap();

        let stats = registry.stats();
        assert_eq!(stats.actor_count, 2);
        assert!(stats.total_size_bytes > 0);
    }
}
