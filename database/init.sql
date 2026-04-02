-- C:\Users\rajku\Downloads\sem 6\AC applied cryptography\projects\PROJECT_SENTINEL_V4\SENTINEL_CBDC_PROJECT\database\init.sql

-- 1. EXTENSIONS
-- Required for generating random UUIDs inside the database
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- 2. USERS TABLE (The Identity Layer)
-- Stores hashed credentials. Status ENUM controls the 'Kill Switch'.
CREATE TABLE IF NOT EXISTS users (
    user_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    username TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    role VARCHAR(20) NOT NULL DEFAULT 'user',       -- 'admin', 'user', 'auditor'
    status TEXT NOT NULL DEFAULT 'ACTIVE',           -- 'ACTIVE', 'FROZEN', 'SUSPENDED'
    is_frozen BOOLEAN NOT NULL DEFAULT false,        -- Quick boolean for freeze checks
    created_at TIMESTAMP DEFAULT NOW()
);

-- 3. TOKENS TABLE (The UTXO Ledger)
-- This is the Money. Each row is a specific digital note.
-- There is no "Account Balance" column in this system; balance is calculated dynamically.
CREATE TABLE IF NOT EXISTS tokens (
    token_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    owner_id UUID REFERENCES users(user_id) NOT NULL,
    value BIGINT NOT NULL CHECK (value > 0),
    status TEXT NOT NULL DEFAULT 'ACTIVE', -- 'ACTIVE', 'SPENT' (Burnt tokens remain for audit)
    created_at TIMESTAMP DEFAULT NOW()
);

-- 4. TRANSACTIONS TABLE (The Forensic Audit Trail)
-- Immutable append-only log of every state change.
-- 'tx_id' links the input tokens (SPENT) to output tokens (ACTIVE).
CREATE TABLE IF NOT EXISTS transactions (
    tx_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tx_type TEXT NOT NULL, -- 'MINT', 'TRANSFER', 'DEFRAG'
    from_user UUID,        -- Nullable for MINT operations
    to_user UUID,          -- Nullable for BURN operations
    amount BIGINT NOT NULL,
    timestamp TIMESTAMP DEFAULT NOW()
);

-- 5. DEFRAG HISTORY TABLE (UTXO Consolidation Audit Trail)
-- Records every defragmentation event for forensic transparency.
CREATE TABLE IF NOT EXISTS defrag_history (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID REFERENCES users(user_id) NOT NULL,
    triggered_by UUID REFERENCES users(user_id),     -- Admin who triggered it (NULL = scheduled)
    triggered_at TIMESTAMP DEFAULT NOW(),
    old_utxo_count INT NOT NULL DEFAULT 0,
    new_utxo_count INT NOT NULL DEFAULT 0,
    total_amount BIGINT NOT NULL DEFAULT 0,
    status TEXT NOT NULL DEFAULT 'COMPLETED',         -- 'COMPLETED', 'FAILED'
    error_message TEXT
);

-- 6. SYSTEM CONFIG TABLE (Runtime Parameters)
-- Key-value store for system-wide settings like kill switch.
CREATE TABLE IF NOT EXISTS system_config (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at TIMESTAMP DEFAULT NOW()
);

-- Insert default config values
INSERT INTO system_config (key, value) VALUES 
    ('transactions_enabled', 'true')
ON CONFLICT (key) DO NOTHING;

-- 7. OPTIMIZATION INDEXES
-- Critical for the 'Knapsack' algorithm to find unspent coins quickly.
CREATE INDEX IF NOT EXISTS idx_tokens_owner_status ON tokens(owner_id, status);
-- Critical for the 'Audit' module to trace user history.
CREATE INDEX IF NOT EXISTS idx_transactions_from_user ON transactions(from_user);
CREATE INDEX IF NOT EXISTS idx_transactions_to_user ON transactions(to_user);
-- Defrag history lookup by user
CREATE INDEX IF NOT EXISTS idx_defrag_history_user ON defrag_history(user_id);
-- Users by role for admin queries
CREATE INDEX IF NOT EXISTS idx_users_role ON users(role);