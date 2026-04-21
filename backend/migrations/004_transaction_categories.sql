-- Create transaction_categories table
CREATE TABLE transaction_categories (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  name TEXT NOT NULL UNIQUE,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Insert default categories
INSERT INTO transaction_categories (name) VALUES
  ('Membership Fees'),
  ('Travel'),
  ('Supplies'),
  ('Events'),
  ('Utilities'),
  ('Rent'),
  ('Other');
