// Library exports for integration tests
pub mod error;
pub mod finance {
    pub mod bank_account;
    pub mod transaction;
    pub mod reconciliation;
    pub mod finance_role;
    pub mod finance_auth;

    pub use bank_account::BankAccount;
    pub use transaction::Transaction;
    pub use reconciliation::Reconciliation;
    pub use finance_role::{FinanceRole, AdminFinanceRole};
}
pub mod storage;
