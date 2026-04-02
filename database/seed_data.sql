-- C:\Users\rajku\Downloads\sem 6\AC applied cryptography\projects\PROJECT_SENTINEL_V4\SENTINEL_CBDC_PROJECT\database\seed_data.sql

-- Initial Data for Testing the Prototype
-- Passwords: admin=Admin@123, alice=Alice@123, bob=Bob@1234

-- 1. Create Users (with role and is_frozen columns)
INSERT INTO users (user_id, username, password_hash, role, status, is_frozen) VALUES 
('a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a11', 'admin', '$argon2id$v=19$m=19456,t=2,p=1$Y60UlAwYEWBmm3QKhMe7KA$cBqCz2KEPJ16b0UyHj+HoG1nVj5JV356dIUgzZNtkeo', 'admin', 'ACTIVE', false),
('b0eebc99-9c0b-4ef8-bb6d-6bb9bd380b22', 'alice', '$argon2id$v=19$m=19456,t=2,p=1$FTWhppuARmRDpSHJvTUicA$a9u+YBfVcBOCuGV8UAWrtA3PphnZAIDmPRoxFX9ik0g', 'user', 'ACTIVE', false),
('c0eebc99-9c0b-4ef8-bb6d-6bb9bd380c33', 'bob',   '$argon2id$v=19$m=19456,t=2,p=1$jb6KUDN1moXfOMrA4/TUZg$DatRZ5O4sTWZZYn9KuaHDnWof7Kg7/s3a5cght7MXyk', 'user', 'ACTIVE', false)
ON CONFLICT (user_id) DO NOTHING;

-- 2. Mint Initial UTXOs for Alice (Total: 500)
-- She gets 5 notes of 100 each.
INSERT INTO tokens (owner_id, value, status) VALUES 
('b0eebc99-9c0b-4ef8-bb6d-6bb9bd380b22', 100, 'ACTIVE'),
('b0eebc99-9c0b-4ef8-bb6d-6bb9bd380b22', 100, 'ACTIVE'),
('b0eebc99-9c0b-4ef8-bb6d-6bb9bd380b22', 100, 'ACTIVE'),
('b0eebc99-9c0b-4ef8-bb6d-6bb9bd380b22', 100, 'ACTIVE'),
('b0eebc99-9c0b-4ef8-bb6d-6bb9bd380b22', 100, 'ACTIVE');

-- 3. Log the Minting Event
INSERT INTO transactions (tx_type, to_user, amount) VALUES 
('MINT', 'b0eebc99-9c0b-4ef8-bb6d-6bb9bd380b22', 500);