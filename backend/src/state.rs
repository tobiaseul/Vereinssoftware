use sqlx::PgPool;
use tokio::sync::broadcast;
use crate::config::Config;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub config: Config,
    pub ws_tx: broadcast::Sender<String>,
}
