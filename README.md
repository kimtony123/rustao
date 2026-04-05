# rustao – Rust SDK for AO

[![Crates.io](https://img.shields.io/crates/v/rustao.svg)](https://crates.io/crates/rustao)
[![docs.rs](https://img.shields.io/docsrs/rustao)](https://docs.rs/rustao)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

**rustao** is an asynchronous Rust library for interacting with the **AO protocol** on Arweave. It provides a clean, ergonomic API to send messages, spawn processes, perform dry runs, query compute endpoints, and handle hybrid encryption – matching the functionality of the official `goao` SDK.

---

## 🚀 Features

- ✅ **Async/Await** – Built on `tokio` for high performance.
- 📨 **Send Messages** – Sign and send messages to AO processes.
- 🌱 **Spawn Processes** – Deploy new processes from module transactions.
- 🧪 **Dry Runs** – Simulate message execution without committing.
- 📊 **Compute Queries** – Fetch process state via HTTP compute endpoints.
- 🔁 **Wait for Results** – Poll until a message result is available.
- 🔐 **Hybrid Encryption** – RSA‑OAEP + AES‑GCM for secure payloads.
- 🧾 **Arweave Wallet Support** – Sign using JWK files.
- 🔑 **Local Crypto** – Embedded RSA signing (from arweave-rust) for security.

---

## 📦 Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
rustao = { path = "../rustao" }
tokio = { version = "1", features = ["full"] }
serde_json = "1.0"
```

---

## 🏗️ Modules

### `rustao::crypto`
Local RSA cryptographic operations (cloned from arweave-rust for security):
- `Driver::generate_key()` – Generate new RSA-PSS key pair
- `PrivateKey::sign(data)` – Sign data with RSA-PSS
- `PrivateKey::verify(data, signature)` – Verify signature

### `rustao::core`
Core Arweave types:
- `TransactionData` – Transaction data structure

### `rustao::Client`
Main client for interacting with AO:
- `send_message()` – Send messages to processes
- `spawn_process()` – Spawn new processes
- `dry_run()` – Simulate execution
- `get_compute_json()` – Query process state

### `rustao::ARSigner`
Wallet signing:
- `ARSigner::from_file(path)` – Load from JWK file
- `sign(data)` – Sign data
- `address()` – Get wallet address

---

## 🚀 Quick Start

```rust
use rustao::{Client, ARSigner, Tag};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load your Arweave wallet (JWK file)
    let signer = ARSigner::from_file("wallet.json")?;

    // Create a client
    let client = Client::new(signer);

    // The process you want to interact with
    let process_id = "6wqH8ue2-bnJG7j--FV0KGYzSs53ObFDofDITb7qtxI";

    // Send a simple message
    let msg_id = client.send_message(
        process_id,
        b"{\"action\": \"hello\"}",
        vec![Tag::new("Action", "Message")],
        None,
        None,
    ).await?;
    println!("Message sent: {}", msg_id);

    Ok(())
}
```

---

## 🧪 Testing

Run all tests:
```bash
cargo test
```

Run live network tests:
```bash
cargo test test_send_message -- --ignored
```

---

## 📱 Example Dapp

See `/home/tony/rustao-example` for a complete Dapp that fetches dApps from AoStore:

```bash
cd rustao-example
cargo run
```

Then visit `http://localhost:8080`

---

## 🧑‍💻 Contributing

Contributions are welcome! Please open an issue or pull request.

---

## License

MIT License. See the LICENSE file for details.

---

## 🙏 Acknowledgements

- Built on top of [arweave-rust](https://github.com/kimtony123/arweave-rust) for cryptographic operations
- Inspired by [goao](https://github.com/ar-aostore/goao)
