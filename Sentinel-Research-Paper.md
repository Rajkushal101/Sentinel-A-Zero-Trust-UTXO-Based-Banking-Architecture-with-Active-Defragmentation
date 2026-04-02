---
title: "Sentinel: Optimizing UTXO Data Structures in Central Bank Digital Currencies via Active Defragmentation"
subtitle: "Backend Validation, Cross-Platform Ecosystem, and Cryptographic Security in Modern CBDCs"
author: Rajkumar
date: March 2026
---

# Abstract

The digitization of sovereign currencies into Central Bank Digital Currencies (CBDCs) demands extreme security, privacy architectures, and friction-less user experiences. While Unspent Transaction Output (UTXO) models provide superior audibility and prevent double-spending, they suffer from "wallet fragmentation"—an accumulation of microscopic, unspent tokens (dust) that degrade system performance over time. This paper presents **Sentinel**, a robust CBDC platform built to evaluate modern cryptographic security, Zero-Trust principles, and a UTXO-based token ledger. Crucially, this paper details Sentinel's **Active Defragmentation Engine**, a vital protocol ensuring that mobile clients and central databases maintain high-throughput efficiency without sacrificing cryptographic integrity. The paper further explores the high-performance **Rust backend (Axum)**, the resilient **PostgreSQL** database, and the integrated **Android mobile application**.

---

# 1. Introduction

With ongoing global research into Central Bank Digital Currencies (CBDCs), architectural decisions surrounding privacy, speed, and systemic security have become paramount. Unlike decentralized blockchains, a retail CBDC typically operates on a centralized ledger managed by the central bank or delegated commercial entities. 

While account-balance models are simple, Central Banks increasingly favor the **UTXO (Unspent Transaction Output)** model to trace the exact lineage of digital currency. However, UTXO structures cause a systemic flaw known as "fragmentation." Every time a user spends a portion of a token, change is created as a new, smaller token. Over thousands of transactions, a user's wallet becomes clogged with fragmented "dust" tokens.

Project **Sentinel** was developed to prototype a production-grade CBDC ecosystem prioritizing **data immutability, cryptographic session isolation, administrative transparency, and structural data optimization**. The ecosystem consists of:
1. **Sentinel Core**: The high-performance Rust backend serving as the central ledger.
2. **Sentinel Vault**: A PostgreSQL relational database structured around UTXO semantics.
3. **Client Ecosystem**: An Android mobile application and web-based banking portal engineered to securely interface with the Core.

---

# 2. The Core Challenge: UTXO Fragmentation

In a UTXO-based CBDC, money does not exist as a simple integer indicating "Account Balance = $100". Instead, money exists as discrete, cryptographic tokens, each with a unique `token_id`, an `owner_id`, and a `value`.

When Alice sends $10 to Bob, if Alice only holds a $50 token, the system must:
1. Cryptographically "burn" (mark as SPENT) Alice's $50 token.
2. Mint a new $10 token for Bob.
3. Mint a $40 "change" token back to Alice.

### 2.1 The Degradation Curve
If Alice makes twenty $1 purchases, her wallet will contain 20 fragmented change tokens. When Alice attempts to make a large $20 purchase, the mobile application and the backend database must gather, sum, and process 20 distinct database rows and cryptographic signatures. 
For a Central Bank processing millions of transactions per second across a nation, this fragmentation causes massive database locking contention, massive I/O overhead, and severe latency spikes on low-bandwidth mobile networks querying their balances.

---

# 3. Algorithm Verification: The Sentinel Defragmentation Engine

To resolve the existential threat of UTXO fragmentation, Sentinel shifts away from passive ledgering and implements an **Active Defragmentation Engine** (`defrag.rs`). This protocol allows wallets to optimize their data structures mathematically, merging "dust" into unified singular tokens.

This is the central research achievement of the Sentinel project. The defragmentation method is handled entirely by the Rust backend to ensure atomic consistency.

## 3.1 The Algorithmic Flow of Defragmentation

When optimization is triggered (either manually by the user, scheduled via cron, or initiated system-wide by the Central Bank admin), the Rust engine executes **Algorithm C: Wallet Optimization**:

1. **Pessimistic Locking (`SELECT ... FOR UPDATE`)**: 
   The initial and most critical step is concurrency control. The algorithm begins a rigid `sqlx` transaction (`pool.begin()`) and searches for all active tokens belonging to the user. It appends the `FOR UPDATE` SQL clause, locking these specific rows in the PostgreSQL database. This ensures that if the user attempts to send money via the mobile app concurrently, the transaction will stall until defragmentation completes, mathematically eliminating the possibility of a "double-spend" during optimization.

2. **Summation and Verification**:
   The engine aggregates the tokens into memory. If the user possesses fewer than two tokens, the engine immediately self-terminates the transaction, logging a "no-op" to save database resources. Otherwise, it calculates the `total_value` of all accumulated dust.

3. **Status Mutability (The Burning Phase)**:
   A standard block-chain burns tokens by deleting them or sending them to a null address. In a central auditing CBDC, data destruction is illegal. Therefore, Sentinel iteratively traverses the locked UTXOs and updates their SQL status from `ACTIVE` to `CONSOLIDATED`. This renders them inert for spending but permanently visible to forensics.

4. **Atomic Minting**:
   A single, new cryptographic UUID (`token_id`) is generated. The system inserts this singular token back into the ledger with the status `ACTIVE` and the exact value of the `total_value` summation. 

5. **Ledger Receipt Generation**:
   The `transactions` table requires a double-entry ledger format. The engine registers a transaction of type `DEFRAG` where the sender and receiver are the *same user ID*, transferring the `total_value` to themselves.

6. **Transaction Context Commit**:
   Because steps 1 through 5 run inside an enclosed database transaction context, any structural failure (e.g., loss of database connection, data type overflow) triggers an immediate `ROLLBACK`. Tokens are either fully consolidated, or untouched. No value is ever lost in limbo.

7. **Asynchronous Audit Logging**:
   Once the transaction strictly commits, an asynchronous spawned task writes the metric statistics (e.g., *User Alice compressed 45 tokens into 1 token*) into the `defrag_history` table. This allows Central Bank operators to macro-analyze network health.

## 3.2 System-Wide Admin Defragmentation
The architecture also features a global endpoint (`optimize_all_wallets`). During off-peak hours (e.g., 3:00 AM), Central Bank super-administrators can trigger a system-wide loop that iterates through every active user in the nation, running the defragmentation mechanism on each wallet. This ensures the national database remains constantly compressed and operational at high speed.

---

# 4. Cryptographic Security & Zero-Trust Architecture

Sentinel operates on a Zero-Trust architecture, assuming the network is inherently hostile. Security mechanisms surround the core ledger.

## 4.1 Authentication and Device Binding
Sentinel utilizes JSON Web Tokens (JWT) for stateless authentication. However, recognizing the vulnerability of standard JWTs to theft, Sentinel implements **Device Binding**:
- Upon login, a unique `device_fingerprint` is generated and heavily bound to the JWT payload.
- If a token is stolen and replayed from an unauthorized device or IP configuration, the system rejects the transaction.
- Passwords are never stored in plaintext. They are hashed using **Argon2id**, the current competitive standard for memory-hard key derivation.

## 4.2 Secure Client-Side Storage (AES-256-GCM)
A critical vulnerability in web applications is the storage of sensitive session tokens in LocalStorage or SessionStorage, susceptible to Cross-Site Scripting (XSS). 
Sentinel solves this by dynamically generating an **ephemeral AES-256-GCM key** per browser session.
1. When the frontend initializes, it generates a WebCrypto AES key.
2. The key is exported as a JWK (JSON Web Key) and safely maintained within the active tab memory (SessionStorage).
3. The actual JWT and user roles are encrypted with this AES key before being stored on the client. 
4. Even if an attacker dumps the browser storage, they retrieve only AES-encrypted ciphertext without access to the decryption key context.

---

# 5. Mobile Ecosystem Integration

Simultaneously developed with the core project is the **Sentinel Android Mobile Application**. Extending the CBDC reach from desktops to smartphones introduces distinct security and networking challenges.

## 5.1 Mobile Application Architecture & Foreground Services
The Android app is built to communicate seamlessly with the Rust backend. To ensure reliable P2P payments in retail environments, connection stability is crucial. The Android ecosystem utilizes strict **Foreground Services**. This ensures that the operating system memory manager does not arbitrarily suspend active transaction routines (or defragmentation requests) during critical verification handshakes with the Sentinel Core.

## 5.2 Optimizing Mobile Bandwidth via Defragmentation
The direct beneficiary of the Defragmentation Engine is the mobile ecosystem. If a mobile user continuously receives micro-payments (e.g., a merchant), loading their transaction history and querying their balance would require pulling thousands of `tokens` over a cellular network. By natively integrating a `[Defragment Wallet]` button into the Android UI, users can compress their ledger states on-demand. This reduces the JSON payload fetched over the network from megabytes to kilobytes, drastically reducing cellular data costs and UI rendering latency.

## 5.3 Zero-Trust Transfer Flows
When executing a transfer, the mobile app requires explicit recipient validation. It presents an immutable local ledger detailing the exact transition of funds, backed by cryptographic receipts returned by the Axum server. 

---

# 6. Administrative Controls & System Diagnostics

A central bank ledger requires absolute administrative authority mixed with extreme observability.

## 6.1 Kill Switch and Account Freezing
- **Global Kill Switch**: By modifying the `system_config` table (Key: `transactions_enabled = false`), the system immediately halts all peer-to-peer transfers worldwide.
- **Account Freezing**: Individual users suspected of malicious activity can have their `is_frozen` status changed to true, instantly barring them from transactions.

## 6.2 The Diagnostics Engine
Sentinel features a deep diagnostics engine (`diagnostics.html`). Upon triggering the `/api/admin/diagnostics` endpoint, the system analyzes critical vectors including PostgreSQL latency, the structural integrity of all 5 core tables, parity between Active/Spent UTXOs vs Total Macroeconomic Supply, and scans for "Orphaned" tokens (tokens disconnected from valid users). If any check registers a malfunction, operators are alerted with CRITICAL badges.

---

# 7. Conclusion

The Sentinel CBDC ecosystem provides a detailed blueprint for digital sovereign money. By isolating logic into a memory-safe Rust backend, adopting a highly auditable UTXO ledger, and enforcing AES-encrypted client sessions, Sentinel achieves a high apex of security. 

Most importantly, the **Defragmentation Engine** solves the greatest structural flaw of UTXO models—data fragmentation. By leveraging pessimistic SQL locking and atomic status mutations, Sentinel dramatically reduces server overhead and cellular data payloads for mobile users. As central banks transition from research to active real-world pilot testing, architectures akin to Sentinel—which balance infrastructural authority (Kill Switches) with rigorous data-structure optimization—will serve as foundational keystones for the future of decentralized-centralized finance.
