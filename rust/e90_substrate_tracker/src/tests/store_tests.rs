use crate::domain::model::{SubstrateHistory, SubstrateId, SubstrateLocation};
use crate::store::TrackingStore;

#[tokio::test]
async fn test_track_and_get_history() {
    let store = TrackingStore::new();
    let substrate_id = "sub-001".to_string();
    let location = SubstrateLocation {
        substrate: SubstrateId {
            carrier_id: "c-001".to_string(),
            slot_number: 1,
            substrate_id: substrate_id.clone(),
        },
        location_id: "lp-001".to_string(),
        location_type: "loadport".to_string(),
        timestamp: None,
    };
    
    store.track(substrate_id.clone(), location).await;
    
    let history = store.get_history(&substrate_id).await.unwrap();
    assert_eq!(history.substrate.substrate_id, substrate_id);
    assert_eq!(history.locations.len(), 1);
    assert_eq!(history.locations[0].location_id, "lp-001");
}

#[tokio::test]
async fn test_multiple_locations() {
    let store = TrackingStore::new();
    let substrate_id = "sub-001".to_string();
    
    store.track(substrate_id.clone(), SubstrateLocation {
        substrate: SubstrateId {
            carrier_id: "c-001".to_string(),
            slot_number: 1,
            substrate_id: substrate_id.clone(),
        },
        location_id: "lp-001".to_string(),
        location_type: "loadport".to_string(),
        timestamp: None,
    }).await;
    
    store.track(substrate_id.clone(), SubstrateLocation {
        substrate: SubstrateId {
            carrier_id: "c-001".to_string(),
            slot_number: 1,
            substrate_id: substrate_id.clone(),
        },
        location_id: "chamber-001".to_string(),
        location_type: "chamber".to_string(),
        timestamp: None,
    }).await;
    
    let history = store.get_history(&substrate_id).await.unwrap();
    assert_eq!(history.locations.len(), 2);
    assert_eq!(history.locations[1].location_id, "chamber-001");
}

#[tokio::test]
async fn test_get_history_not_found() {
    let store = TrackingStore::new();
    let history = store.get_history("nonexistent").await;
    assert!(history.is_none());
}

#[tokio::test]
async fn test_batch_history() {
    let store = TrackingStore::new();
    
    store.track("sub-001".to_string(), SubstrateLocation {
        substrate: SubstrateId { carrier_id: "c-001".to_string(), slot_number: 1, substrate_id: "sub-001".to_string() },
        location_id: "lp-001".to_string(),
        location_type: "loadport".to_string(),
        timestamp: None,
    }).await;
    
    store.track("sub-002".to_string(), SubstrateLocation {
        substrate: SubstrateId { carrier_id: "c-001".to_string(), slot_number: 2, substrate_id: "sub-002".to_string() },
        location_id: "lp-001".to_string(),
        location_type: "loadport".to_string(),
        timestamp: None,
    }).await;
    
    let histories = store.get_batch_history(&["sub-001".to_string(), "sub-002".to_string()]).await;
    assert_eq!(histories.len(), 2);
}
