# Deployment Guide

**Target:** Linux server (Ubuntu/Debian), single machine running Nginx + PostgreSQL + the Rust backend.

---

## Prerequisites

Install the required packages on the server:

```bash
sudo apt update
sudo apt install -y postgresql nginx certbot python3-certbot-nginx
```

Install the Rust toolchain (needed to compile the backend on the server, or see the cross-compile alternative below):

```bash
curl https://sh.rustup.rs -sSf | sh
source ~/.cargo/env
```

Install Node.js (only needed to build the frontend — can be done locally):

```bash
curl -fsSL https://deb.nodesource.com/setup_22.x | sudo -E bash -
sudo apt install -y nodejs
```

---

## 1. Database setup

```bash
sudo -u postgres psql <<'EOF'
CREATE USER vereinssoftware WITH PASSWORD 'your-db-password';
CREATE DATABASE vereinssoftware OWNER vereinssoftware;
EOF
```

---

## 2. Copy the project to the server

```bash
# On your local machine
rsync -av --exclude target --exclude node_modules \
  /Users/tobi/Documents/Coding/Vereinssoftware/ \
  user@your-server:/opt/vereinssoftware/
```

---

## 3. Configure environment

```bash
# On the server
cd /opt/vereinssoftware
cp .env.example .env   # or create .env manually
nano .env
```

Set these values:

```env
DATABASE_URL=postgres://vereinssoftware:your-db-password@localhost:5432/vereinssoftware
JWT_SECRET=replace-this-with-a-long-random-string   # e.g. openssl rand -hex 64
JWT_EXPIRY_SECONDS=900
REFRESH_TOKEN_EXPIRY_DAYS=7
BACKEND_PORT=3000
FRONTEND_URL=https://your-domain.com
```

Generate a strong `JWT_SECRET`:

```bash
openssl rand -hex 64
```

---

## 4. Run database migrations

Install `sqlx-cli` if not already installed:

```bash
cargo install sqlx-cli --no-default-features --features postgres
```

Run the migrations:

```bash
cd /opt/vereinssoftware/backend
sqlx migrate run --database-url "postgres://vereinssoftware:your-db-password@localhost:5432/vereinssoftware"
```

---

## 5. Seed the first SuperAdmin

```bash
cd /opt/vereinssoftware/backend
cargo run --bin seed -- --username admin --password 'choose-a-strong-password' --role SuperAdmin
```

Change the password after first login via the Admin Users settings page.

---

## 6. Build the backend

```bash
cd /opt/vereinssoftware/backend
cargo build --release
```

The binary is at `target/release/vereinssoftware-backend`.

---

## 7. Build the frontend

```bash
cd /opt/vereinssoftware/frontend
npm ci
npm run build
```

The built files are in `frontend/dist/`. Copy them to the web root:

```bash
sudo mkdir -p /var/www/vereinssoftware
sudo cp -r /opt/vereinssoftware/frontend/dist/* /var/www/vereinssoftware/
```

---

## 8. Run the backend as a systemd service

Create `/etc/systemd/system/vereinssoftware.service`:

```ini
[Unit]
Description=Vereinssoftware Backend
After=network.target postgresql.service

[Service]
Type=simple
User=www-data
WorkingDirectory=/opt/vereinssoftware/backend
EnvironmentFile=/opt/vereinssoftware/.env
ExecStart=/opt/vereinssoftware/backend/target/release/vereinssoftware-backend
Restart=on-failure
RestartSec=5

[Install]
WantedBy=multi-user.target
```

Enable and start it:

```bash
sudo systemctl daemon-reload
sudo systemctl enable vereinssoftware
sudo systemctl start vereinssoftware
sudo systemctl status vereinssoftware
```

---

## 9. Configure Nginx

Create `/etc/nginx/sites-available/vereinssoftware`:

```nginx
server {
    listen 80;
    server_name your-domain.com;

    root /var/www/vereinssoftware;
    index index.html;

    # API and auth routes → backend
    location ~ ^/(api|auth|ws) {
        proxy_pass http://localhost:3000;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }

    # Frontend — serve index.html for all other routes (SPA)
    location / {
        try_files $uri $uri/ /index.html;
    }
}
```

Enable the site:

```bash
sudo ln -s /etc/nginx/sites-available/vereinssoftware /etc/nginx/sites-enabled/
sudo nginx -t
sudo systemctl reload nginx
```

---

## 10. HTTPS (recommended)

```bash
sudo certbot --nginx -d your-domain.com
```

Certbot will update the Nginx config automatically and set up auto-renewal.

---

## Updating the app

When you push changes:

```bash
# On the server
cd /opt/vereinssoftware

# Pull latest code (or re-rsync from local)
git pull   # if you have git set up on the server

# Rebuild backend (if backend changed)
cd backend && cargo build --release
sudo systemctl restart vereinssoftware

# Rebuild frontend (if frontend changed)
cd ../frontend && npm ci && npm run build
sudo cp -r dist/* /var/www/vereinssoftware/

# Run new migrations (if any)
sqlx migrate run --database-url "$DATABASE_URL"
```
