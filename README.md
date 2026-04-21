# Vereinssoftware

A comprehensive association management system built with Rust and Vue 3. Manage members, finances, transactions, and organizational workflows with ease.

## Features

### Member Management
- Create, update, and delete member profiles
- Track membership status (Aktiv, Passiv, Ehrenmitglied)
- Custom fields support for flexible data collection
- Member search and filtering
- Export member data

### Financial Management
- **Bank Account Management**: Create and manage multiple bank accounts with IBAN support
- **Transaction Tracking**: Record income, expenses, transfers, and refunds with detailed categorization
- **Global Transactions Page**: View and filter all transactions across accounts by type, category, date, member, and reconciliation status
- **Configurable Categories**: Define and manage transaction categories via Settings
- **Receipt Management**: Upload and attach receipts to transactions (JPG, PNG, PDF up to 10MB)
- **Reconciliation**: Match bank statements with recorded transactions

### Admin Management
- SuperAdmin and Admin roles
- Finance role assignments (Treasurer, Finance Officer)
- User creation and password management
- Role-based access control (RBAC)

### Configuration
- **Organized Settings**: Centralized Settings page with Configuration subpages
- **Member Fields**: Define custom member fields (text, number, date, boolean, enum)
- **Transaction Categories**: Manage transaction categories from Settings
- **Field Options**: Create and manage enum field options

## Tech Stack

### Backend
- **Language**: Rust
- **Framework**: Axum (async web framework)
- **Database**: PostgreSQL with SQLx for type-safe queries
- **Authentication**: JWT tokens with refresh mechanism
- **WebSocket**: Real-time presence and notifications
- **File Storage**: Local filesystem with receipt management

### Frontend
- **Framework**: Vue 3 (Composition API)
- **Language**: TypeScript
- **UI Library**: Element Plus
- **State Management**: Pinia
- **API Client**: Axios with custom middleware
- **Data Fetching**: TanStack Vue Query
- **Routing**: Vue Router with auth guards
- **Build Tool**: Vite

## Project Structure

```
├── backend/
│   ├── src/
│   │   ├── admins/          # Admin management
│   │   ├── auth/            # Authentication & JWT
│   │   ├── finance/         # Finance domain
│   │   │   ├── bank_account.rs
│   │   │   ├── category.rs
│   │   │   ├── handlers.rs
│   │   │   └── transaction.rs
│   │   ├── members/         # Member management
│   │   ├── field_definitions/ # Custom fields
│   │   ├── services/        # Business logic services
│   │   └── main.rs
│   ├── migrations/          # Database migrations
│   └── Cargo.toml
├── frontend/
│   ├── src/
│   │   ├── api/            # API client functions
│   │   ├── components/     # Reusable Vue components
│   │   ├── stores/         # Pinia state stores
│   │   ├── views/          # Page components
│   │   ├── router/         # Vue Router configuration
│   │   ├── types/          # TypeScript types
│   │   └── App.vue
│   └── package.json
└── docs/                    # Documentation
```

## Setup & Installation

### Prerequisites
- Rust 1.70+
- Node.js 18+
- PostgreSQL 14+
- pnpm (recommended) or npm

### Backend Setup

```bash
cd backend

# Set up environment
cp .env.example .env
# Edit .env with your database credentials

# Install dependencies & build
cargo build

# Run migrations
sqlx migrate run

# Start the server
cargo run
```

The backend will run on `http://localhost:3000` (or port from `PORT` env var).

### Frontend Setup

```bash
cd frontend

# Install dependencies
pnpm install

# Set up environment
cp .env.example .env
# Update VITE_API_URL if backend is on different address

# Start dev server
pnpm dev

# Build for production
pnpm build
```

The frontend will run on `http://localhost:5173`.

## API Endpoints

### Authentication
- `POST /auth/login` - Login with username/password
- `POST /auth/refresh` - Refresh JWT token
- `POST /auth/logout` - Logout

### Members
- `GET /api/v1/members` - List all members
- `POST /api/v1/members` - Create member
- `GET /api/v1/members/:id` - Get member details
- `PUT /api/v1/members/:id` - Update member
- `DELETE /api/v1/members/:id` - Delete member
- `GET /api/v1/members/export` - Export members as CSV

### Bank Accounts
- `GET /api/v1/finance/accounts` - List accounts
- `POST /api/v1/finance/accounts` - Create account (Treasurer role)
- `GET /api/v1/finance/accounts/:id` - Get account details
- `PUT /api/v1/finance/accounts/:id` - Update account (Treasurer role)
- `DELETE /api/v1/finance/accounts/:id` - Delete account (Treasurer role)

### Transactions
- `GET /api/v1/finance/transactions` - List all transactions with filters
- `GET /api/v1/finance/accounts/:id/transactions` - List account transactions
- `POST /api/v1/finance/accounts/:id/transactions` - Create transaction (Finance Officer role)
- `GET /api/v1/finance/transactions/:id` - Get transaction details
- `PUT /api/v1/finance/transactions/:id` - Update transaction (Finance Officer role)
- `DELETE /api/v1/finance/transactions/:id` - Delete transaction (Finance Officer role)
- `POST /api/v1/finance/transactions/:id/receipt` - Upload receipt
- `GET /api/v1/finance/transactions/:id/receipt/:ref` - Download receipt

### Transaction Categories
- `GET /api/v1/finance/categories` - List categories
- `POST /api/v1/finance/categories` - Create category (SuperAdmin only)
- `DELETE /api/v1/finance/categories/:id` - Delete category (SuperAdmin only)

### Admin Management
- `GET /api/v1/admins` - List admins
- `POST /api/v1/admins` - Create admin
- `DELETE /api/v1/admins/:id` - Delete admin
- `PUT /api/v1/admins/:id/password` - Change password
- `GET /api/v1/finance/admins/:id/roles` - Get finance roles for admin
- `POST /api/v1/finance/admins/:id/roles` - Assign finance role
- `DELETE /api/v1/finance/admins/:id/roles/:role` - Remove finance role

## Architecture

### Authentication Flow
1. User logs in with username/password
2. Backend verifies credentials and issues JWT token + refresh token
3. Frontend stores access token in memory, refresh token in HTTP-only cookie
4. All subsequent API requests include JWT in Authorization header
5. On token expiry, frontend automatically refreshes using refresh token
6. If refresh fails, user is redirected to login

### Role-Based Access Control
- **SuperAdmin**: Full system access, user/admin management
- **Admin**: Member and field management
- **Treasurer**: Bank account and reconciliation management
- **Finance Officer**: Transaction creation and management

### Finance Domain
- **Bank Accounts**: Independent entities with balance tracking
- **Transactions**: Associated with accounts, support soft/hard deletes
- **Categories**: Configurable transaction categories
- **Reconciliation**: Match bank statements with transactions
- **Receipts**: File storage for transaction attachments

### State Management
- Global state (auth, notifications) in Pinia stores
- Local component state for forms
- Server state cached with TanStack Vue Query
- Real-time updates via WebSocket

## Database Schema

Key tables:
- `admins` - User accounts
- `members` - Association members
- `bank_accounts` - Financial accounts
- `transactions` - Financial transactions
- `transaction_categories` - Transaction types
- `field_definitions` - Custom member fields
- `admin_finance_roles` - Finance role assignments
- `reconciliations` - Bank reconciliation records

See migrations in `backend/migrations/` for complete schema.

## Development

### Running Tests
```bash
cd backend
cargo test

cd ../frontend
pnpm test
```

### Code Style
- Rust: Follow clippy guidelines (`cargo clippy`)
- TypeScript: ESLint configured in `frontend/`
- Both: Format before committing

### Creating Migrations
```bash
cd backend
sqlx migrate add -r <migration_name>
# Edit created files, then run migrations
sqlx migrate run
```

## Deployment

### Docker
```bash
docker-compose up -d
```

### Manual Deployment
1. Build backend: `cd backend && cargo build --release`
2. Build frontend: `cd frontend && pnpm build`
3. Serve frontend files statically
4. Run backend binary with environment variables configured

## Contributing

1. Create a feature branch: `git checkout -b feature/description`
2. Make your changes and write tests
3. Commit with clear messages
4. Push to GitHub and create a Pull Request
5. Ensure CI checks pass before merging

## License

Proprietary - All rights reserved

## Support

For issues, bugs, or feature requests, please open an issue on GitHub.

---

**Built with ❤️ for association management**
