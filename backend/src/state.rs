use sqlx::PgPool;
use tokio::sync::broadcast;
use crate::config::Config;
use std::sync::Arc;
use crate::storage::FileStorage;
use crate::services::reconciliation::ReconciliationService;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub config: Config,
    pub ws_tx: broadcast::Sender<String>,
    pub file_storage: Arc<dyn FileStorage>,
    pub reconciliation_service: Arc<dyn ReconciliationService>,
}
