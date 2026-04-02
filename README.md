<!-- C:\Users\rajku\Downloads\sem 6\AC applied cryptography\projects\PROJECT_SENTINEL_V4\SENTINEL_CBDC_PROJECT\README.md -->

🛡️ Project Sentinel: Sovereign CBDC Architecture

Version: 5.0 (Research Prototype)

Status: Research Ready

Stack: Rust, PostgreSQL, Docker, Vanilla JS

1. Abstract

Project Sentinel is a Hybrid UTXO-based Central Bank Digital Currency (CBDC) system. Unlike traditional Account-based models (e.g., Ethereum), Sentinel utilizes a discrete token model to ensure Atomic State Transitions and Forensic Auditability.

The system introduces a "Zero-Trust Client" framework, shifting security from the network perimeter to the endpoint via Polymorphic CAPTCHAs, Ephemeral Encryption, and DOM-based Anti-Exfiltration measures.

2. Architecture

Layer 1: The Vault (Data Persistence)

Technology: PostgreSQL 16 (Strict Serializability)

Logic: Immutable Append-Only Ledger.

Innovation: Every transaction is cryptographically hashed (prev_hash + data) creating a verifiable audit chain.

Layer 2: The Sovereign Core (Backend)

Technology: Rust (Axum + Tokio)

Logic: * Coin Selection: Knapsack algorithm to optimize UTXO spending.

Atomic Swaps: ACID-compliant transfer engine.

Defragmentation: Automatic merging of "Dust" (small coins).

Layer 3: The Edge Gateway (Security)

Technology: Middleware Filters

Logic: * Stateless CAPTCHA: Animated SVG generation with AES-256 state encryption.

Rate Limiting: Token-bucket IP throttling.

Session Binding: JWTs bound to Device Fingerprints.

Layer 4: The Zero-Trust Client (Frontend)

Technology: Vanilla JS (No Frameworks)

Logic: * Active Defense: Disables Right-Click, PrintScreen, DevTools.

Secure Storage: Uses ephemeral AES keys in RAM to encrypt SessionStorage.

3. Setup & Installation

Prerequisites

Docker & Docker Compose

Rust (Cargo) v1.75+

Quick Start

Initialize Infrastructure:

cd docker
docker-compose up -d


Run the Core:

cd ../core
cargo run


Access the Interface:

User Wallet: http://localhost:3000/ (via Frontend server or file)

API Health: http://localhost:3000/api/admin/stats

4. Running the Research Experiments

Experiment A: Security Penetration

Run the Python attack simulator to test the WAF and Input Sanitization.

python3 scripts/attack_sim.py


Expected Result: All SQL Injection and XSS attempts should return BLOCKED.

Experiment B: Atomic Transfer

Login as alice (pass: pass123).

Navigate to "Send Money".

Send 100 to bob.

Observe the UTXO count decrease and Balance update instantly.

5. Directory Structure

core/src/banking: The UTXO Logic Engine.

core/src/security: The Rust Security Modules.

core/src/audit: The Forensic Logging System.

frontend/common: The Shared Security JS Libraries.

Disclaimer: This is a research prototype designed for academic evaluation of CBDC architectures. Do not use for real value transfer without a formal security audit.