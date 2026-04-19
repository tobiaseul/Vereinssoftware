pub mod manual;

pub use manual::ManualReconciliationService;

use async_trait::async_trait;
use chrono::NaiveDate;
use sqlx::PgPool;
use uuid::Uuid;

pub struct StatementLine {
    pub date: NaiveDate,
    pub amount: String,
    pub reference: String,
}

#[derive(Debug, Clone)]
pub enum ReconciliationError {
    DatabaseError(String),
    ParseError(String),
    NoMatchFound(String),
}

impl std::fmt::Display for ReconciliationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReconciliationError::DatabaseError(msg) => write!(f, "Database Error: {}", msg),
            ReconciliationError::ParseError(msg) => write!(f, "Parse Error: {}", msg),
            ReconciliationError::NoMatchFound(msg) => write!(f, "No Match Found: {}", msg),
        }
    }
}

impl std::error::Error for ReconciliationError {}

#[async_trait]
pub trait ReconciliationService: Send + Sync {
    async fn match_transactions(
        &self,
        statement_lines: Vec<StatementLine>,
        account_id: Uuid,
        pool: &PgPool,
    ) -> Result<Vec<(StatementLine, Option<Uuid>)>, ReconciliationError>;
}
