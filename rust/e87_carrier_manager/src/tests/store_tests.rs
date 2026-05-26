use crate::domain::model::{Carrier, CarrierState, LoadPort, LoadPortTransferState, AccessMode};
use crate::store::CarrierStore;
use semi_common::error::CimError;

#[tokio::test]
async fn test_create_and_get_loadport() {
    let store = CarrierStore::new();
    let lp = LoadPort {
        loadport_id: "lp-001".to_string(),
        transfer_state: LoadPortTransferState::InService,
        access_mode: AccessMode::Auto,
        associated_carrier_id: String::new(),
        reserved: false,
    };
    store.create_loadport(lp.clone()).await.unwrap();

    let found = store.get_loadport("lp-001").await.unwrap();
    assert_eq!(found.loadport_id, "lp-001");
    assert_eq!(found.access_mode, AccessMode::Auto);
}

#[tokio::test]
async fn test_create_and_get_carrier() {
    let store = CarrierStore::new();
    let c = Carrier {
        carrier_id: "c-001".to_string(),
        state: CarrierState::WaitingForCarrier,
        loadport_id: "lp-001".to_string(),
        slot_map: vec!["1".to_string(), "0".to_string()],
        arrived_at: None,
    };
    store.create_carrier(c.clone()).await.unwrap();

    let found = store.get_carrier("c-001").await.unwrap();
    assert_eq!(found.carrier_id, "c-001");
    assert_eq!(found.state, CarrierState::WaitingForCarrier);
}

#[tokio::test]
async fn test_duplicate_carrier_fails() {
    let store = CarrierStore::new();
    let c = Carrier {
        carrier_id: "c-001".to_string(),
        state: CarrierState::WaitingForCarrier,
        loadport_id: "lp-001".to_string(),
        slot_map: vec![],
        arrived_at: None,
    };
    store.create_carrier(c.clone()).await.unwrap();

    let err = store.create_carrier(c).await.unwrap_err();
    assert!(matches!(err, CimError::AlreadyExists(_)));
}

#[tokio::test]
async fn test_set_access_mode() {
    let store = CarrierStore::new();
    let lp = LoadPort {
        loadport_id: "lp-001".to_string(),
        transfer_state: LoadPortTransferState::InService,
        access_mode: AccessMode::Manual,
        associated_carrier_id: String::new(),
        reserved: false,
    };
    store.create_loadport(lp).await.unwrap();

    let updated = store.set_access_mode("lp-001", AccessMode::Auto).await.unwrap();
    assert_eq!(updated.access_mode, AccessMode::Auto);
}

#[tokio::test]
async fn test_verify_slot_map_match() {
    let store = CarrierStore::new();
    let c = Carrier {
        carrier_id: "c-001".to_string(),
        state: CarrierState::Verifying,
        loadport_id: "lp-001".to_string(),
        slot_map: vec!["1".to_string(), "0".to_string()],
        arrived_at: None,
    };
    store.create_carrier(c).await.unwrap();

    let result = store.verify_slot_map("c-001", &["1".to_string(), "0".to_string()]).await.unwrap();
    assert_eq!(result.state, CarrierState::VerifyCompleted);
}

#[tokio::test]
async fn test_verify_slot_map_mismatch() {
    let store = CarrierStore::new();
    let c = Carrier {
        carrier_id: "c-001".to_string(),
        state: CarrierState::Verifying,
        loadport_id: "lp-001".to_string(),
        slot_map: vec!["1".to_string(), "0".to_string()],
        arrived_at: None,
    };
    store.create_carrier(c).await.unwrap();

    let result = store.verify_slot_map("c-001", &["0".to_string(), "1".to_string()]).await.unwrap();
    assert_eq!(result.state, CarrierState::VerifyFailed);
}

#[tokio::test]
async fn test_list_carriers_filter() {
    let store = CarrierStore::new();
    store.create_carrier(Carrier {
        carrier_id: "c-001".to_string(),
        state: CarrierState::WaitingForCarrier,
        loadport_id: "lp-001".to_string(),
        slot_map: vec![],
        arrived_at: None,
    }).await.unwrap();
    store.create_carrier(Carrier {
        carrier_id: "c-002".to_string(),
        state: CarrierState::VerifyCompleted,
        loadport_id: "lp-002".to_string(),
        slot_map: vec![],
        arrived_at: None,
    }).await.unwrap();

    let all = store.list_carriers(None).await;
    assert_eq!(all.len(), 2);

    let waiting = store.list_carriers(Some(CarrierState::WaitingForCarrier)).await;
    assert_eq!(waiting.len(), 1);
}
