-- Finance schema for bank accounts, transactions, reconciliations, and roles

CREATE TABLE bank_accounts (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  name TEXT NOT NULL,
  iban TEXT NOT NULL UNIQUE,
  bank_name TEXT NOT NULL,
  balance DECIMAL(10, 2) NOT NULL DEFAULT 0,
  is_active BOOLEAN NOT NULL DEFAULT true,
  deleted_at TIMESTAMPTZ,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_bank_accounts_deleted_at ON bank_accounts(deleted_at);

CREATE TABLE transactions (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  bank_account_id UUID NOT NULL REFERENCES bank_accounts(id),
  version INTEGER NOT NULL DEFAULT 1,
  type TEXT NOT NULL CHECK (type IN ('Income', 'Expense', 'Transfer', 'Refund')),
  amount DECIMAL(10, 2) NOT NULL CHECK (amount > 0),
  date DATE NOT NULL,
  member_id UUID REFERENCES members(id),
  category TEXT NOT NULL,
  reference TEXT NOT NULL,
  description TEXT,
  receipt_reference TEXT,
  reconciled BOOLEAN NOT NULL DEFAULT false,
  reconciliation_id UUID,
  transfer_pair_id UUID,
  deleted_at TIMESTAMPTZ,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_transactions_bank_account ON transactions(bank_account_id);
CREATE INDEX idx_transactions_member ON transactions(member_id);
CREATE INDEX idx_transactions_reconciliation ON transactions(reconciliation_id);
CREATE INDEX idx_transactions_deleted_at ON transactions(deleted_at);

CREATE TABLE reconciliations (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  bank_account_id UUID NOT NULL REFERENCES bank_accounts(id),
  statement_date DATE NOT NULL,
  file_name TEXT NOT NULL,
  matched_transaction_ids UUID[] NOT NULL DEFAULT '{}',
  status TEXT NOT NULL CHECK (status IN ('InProgress', 'Complete')) DEFAULT 'InProgress',
  uploaded_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_reconciliations_bank_account ON reconciliations(bank_account_id);

CREATE TABLE finance_roles (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  name TEXT NOT NULL UNIQUE,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

INSERT INTO finance_roles (name) VALUES ('Treasurer'), ('Finance Officer');

CREATE TABLE admin_finance_roles (
  admin_id UUID NOT NULL REFERENCES admins(id) ON DELETE CASCADE,
  finance_role_id UUID NOT NULL REFERENCES finance_roles(id),
  assigned_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY (admin_id, finance_role_id)
);

CREATE INDEX idx_admin_finance_roles_admin ON admin_finance_roles(admin_id);

-- Add foreign keys after all tables are created
ALTER TABLE transactions ADD CONSTRAINT fk_transactions_reconciliation FOREIGN KEY (reconciliation_id) REFERENCES reconciliations(id);
ALTER TABLE transactions ADD CONSTRAINT fk_transactions_transfer_pair FOREIGN KEY (transfer_pair_id) REFERENCES transactions(id);
