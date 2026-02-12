use agentkern_energy_ee::grid::GridApi;
use agentkern_governance::esg::GridApi as GridApiTrait;
use agentkern_energy_ee::grid::GridProvider;
use temp_env;

#[tokio::test]
async fn test_grid_api_async_mock() {
    unsafe { std::env::set_var("AGENTKERN_LICENSE_KEY", "PRO-TEST-LICENSE-KEY-1234567890123456789"); }
    
    // 1. Initialize API with mock license
    let api = GridApi::new().expect("Failed to init GridApi"); 
    
    // 2. Test get_intensity async
    let result = api.get_intensity("us-east-1").await;
    assert!(result.is_ok(), "Async get_intensity failed");
    
    let feed = result.unwrap();
    assert_eq!(feed.region, "us-east-1");
    assert!(feed.intensity_gco2_kwh > 0.0, "Intensity should be positive");
}

#[tokio::test]
async fn test_find_greenest_async() {
    unsafe { std::env::set_var("AGENTKERN_LICENSE_KEY", "PRO-TEST-LICENSE-KEY-1234567890123456789"); }

    let api = GridApi::new().expect("Failed to init GridApi");
    
    let regions = vec!["us-east-1", "eu-west-1", "ap-southeast-1"];
    let best = api.find_greenest(&regions).await;
    
    assert!(best.is_ok());
    let best_region = best.unwrap();
    println!("Greenest Region: {}", best_region);
    
    // Mock data: 
    // US-East: 400
    // EU-West: 250 (Should win)
    // AP-SE: 550
    assert_eq!(best_region, "eu-west-1");
}
