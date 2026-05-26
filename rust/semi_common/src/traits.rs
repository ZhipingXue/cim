use async_trait::async_trait;
use crate::error::CimResult;

/// Trait for any SEMI standard service app
#[async_trait]
pub trait SemiService: Send + Sync {
    /// Service type identifier
    fn service_type(&self) -> &'static str;

    /// Initialize the service
    async fn initialize(&self) -> CimResult<()>;

    /// Shutdown the service gracefully
    async fn shutdown(&self) -> CimResult<()>;

    /// Health check
    async fn health(&self) -> CimResult<()>;
}

/// Trait for alarm reporting (distributed across apps, collected by E116)
#[async_trait]
pub trait AlarmReporter: Send + Sync {
    async fn report_alarm(&self, alarm: crate::types::Alarm) -> CimResult<()>;
    async fn clear_alarm(&self, alarm_id: i32) -> CimResult<()>;
}

/// Trait for event publishing
#[async_trait]
pub trait EventPublisher: Send + Sync {
    async fn publish_event(&self, event: crate::types::Event) -> CimResult<()>;
}

/// Trait for state machine reporting (E116 integration)
#[async_trait]
pub trait StateMachineReporter: Send + Sync {
    async fn report_state_transition(
        &self,
        sm_id: String,
        sm_type: String,
        from_state: String,
        to_state: String,
    ) -> CimResult<()>;
}

/// Trait for data publishing to Data Collection service
#[async_trait]
pub trait DataPublisher: Send + Sync {
    async fn publish(&self, key: String, value: String) -> CimResult<()>;
}

/// Trait for service registry client
#[async_trait]
pub trait RegistryClient: Send + Sync {
    async fn register(
        &self,
        endpoint: crate::types::ServiceEndpoint,
    ) -> CimResult<()>;
    async fn deregister(&self,
        service_id: String,
    ) -> CimResult<()>;
    async fn heartbeat(
        &self,
        service_id: String,
    ) -> CimResult<()>;
    async fn discover(
        &self,
        service_type: crate::types::ServiceType,
    ) -> CimResult<Vec<crate::types::ServiceEndpoint>>;
}
