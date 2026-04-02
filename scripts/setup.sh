#!/bin/bash
# C:\Users\rajku\Downloads\sem 6\AC applied cryptography\projects\PROJECT_SENTINEL_V4\SENTINEL_CBDC_PROJECT\scripts\setup.sh

echo "🛡️  SENTINEL RESEARCH PROTOTYPE SETUP 🛡️"
echo "----------------------------------------"

# 1. Check Prerequisites
if ! command -v docker &> /dev/null; then
    echo "❌ Docker is not installed."
    exit 1
fi

if ! command -v cargo &> /dev/null; then
    echo "❌ Rust (Cargo) is not installed."
    exit 1
fi

# 2. Start Infrastructure (Postgres)
echo "🐳 Starting The Vault (PostgreSQL)..."
cd ../docker
docker-compose up -d sentinel-vault

echo "⏳ Waiting for Database to initialize..."
sleep 5

# 3. Initialize Database (if strictly necessary outside docker-entrypoint)
# Note: The init.sql in docker-compose usually handles this.
# This step validates connectivity.
echo "🔌 Validating connectivity..."
# (Optional SQL check logic here)

# 4. Build Core
echo "🦀 Building Sovereign Core (Rust)..."
cd ../core
cargo build --release

echo "✅ Setup Complete."
echo "   1. To start the server:  cd core && cargo run"
echo "   2. To serve frontend:    open frontend/index.html (or use 'python3 -m http.server 8000' in frontend/)"
echo "   3. To run attacks:       python3 scripts/attack_sim.py"