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
- 🧾 **Arweave Wallet Support** – Sign using JWK files or private key hex.

---

## 📦 Installation

Add this to your `Cargo.toml`:

toml
[dependencies]
rustao = { git = "https://github.com/kimtony123/rustao" }
tokio = { version = "1", features = ["full"] }
serde_json = "1.0"
Or using cargo add (if published):

bash
cargo add rustao
🚀 Quick Start
rust
use rustao::{Client, ARSigner};
use rustao::schema::Tag;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load your Arweave wallet (JWK file)
    let signer = ARSigner::from_file("wallet.json")?;

    // Create a client (uses default gateways)
    let client = Client::new().with_signer(signer);

    // The process you want to interact with
    let process_id = "6wqH8ue2-bnJG7j--FV0KGYzSs53ObFDofDITb7qtxI";

    // Send a simple message
    let msg_id = client.send_message(
        process_id,
        b"{\"action\": \"hello\"}",
        vec![Tag::new("Action", "Message")],
        None, // no encryption options
    ).await?;
    println!("Message sent: {}", msg_id);

    // Wait for the result (max 30 seconds, poll every 2 seconds)
    let result = client.wait_for_result(&msg_id, process_id, 30, 2).await?;
    println!("Result: {}", String::from_utf8_lossy(&result.output));

    // Query a compute endpoint
    let counter = client.get_compute_string(process_id, "counter").await?;
    println!("Counter: {}", counter);

    Ok(())
}
# 🧠 Core Concepts
## 🔑 Signer
The ARSigner implements the Signer trait, which provides the cryptographic operations required by the client.

Create from JWK file: ARSigner::from_file("wallet.json")?

Create from private key hex: ARSigner::from_private_key_hex(hex)?

Sign: async fn sign(&self, data: &[u8]) -> Result<Vec<u8>>

Public key: fn public_key(&self) -> Vec<u8> (uncompressed)

Address: fn address(&self) -> String

## 🧩 Client
The Client is the main entry point. It holds the signer and the URLs for the MU (message uploader), CU (compute unit), SU (scheduler), and compute gateway. You can customise these using builder methods:

rust
let client = Client::new()
    .with_signer(signer)
    .with_mu("https://custom-mu.example.com")
    .with_cu("https://custom-cu.example.com")
    .with_su("https://custom-su.example.com")
    .with_compute_gateway("https://custom-gateway.example.com");

## 📨 Messages
Send a message to a process using send_message. Tags are key‑value pairs that provide metadata. The method returns the data item ID (the ID you use to retrieve the result).

rust
let tags = vec![
    Tag::new("Action", "Transfer"),
    Tag::new("Recipient", "some-address"),
];
let msg_id = client.send_message(process_id, b"1000", tags, None).await?;
🌱 Spawn a Process
To create a new process from a module transaction ID:

rust
let process_id = client.spawn_process(
    "module-tx-id",
    b"{\"initial\": \"state\"}",
    vec![Tag::new("App-Name", "MyApp")],
).await?;
##  🧪 Dry Run
Simulate a message without committing it – useful for checking balances or evaluating expressions:

rust
let result = client.dry_run(
    process_id,
    b"{\"action\": \"Balance\"}",
    vec![Tag::new("Action", "Balance")],
    None,
).await?;
println!("Output: {}", String::from_utf8_lossy(&result.output));


## 📊 Compute Queries
AO processes expose state via HTTP compute endpoints. Three helpers are provided:

get_compute(process_id, path) → Vec<u8>

get_compute_string(process_id, path) → String

get_compute_json(process_id, path) → serde_json::Value

rust
let raw = client.get_compute(process_id, "counter").await?;
let str = client.get_compute_string(process_id, "status").await?;
let json = client.get_compute_json(process_id, "state").await?;


## 🔐 Encryption
The SDK supports hybrid encryption (RSA‑OAEP + AES‑GCM) for message payloads. To send an encrypted message, provide a recipient's RSA public key via SendMessageOptions:

rust
use rustao::encrypt::{SendMessageOptions, EncryptWithRSA};

let pub_key = ...; // RSA public key of the recipient process
let opts = SendMessageOptions::new().encrypt_with_rsa(pub_key);

let msg_id = client.send_message(process_id, b"secret data", tags, Some(opts)).await?;
The client automatically encrypts the data, attaches the required tags (Encrypted-Key, Nonce), and sends the encrypted payload. On the receiving side, you can decrypt the response using the DecryptResponse helper.

# 🧪 Full Example
Here’s a complete example that interacts with a live process, adds a product, and fetches all apps:

rust
use rustao::{Client, ARSigner};
use rustao::schema::Tag;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let signer = ARSigner::from_file("wallet.json")?;
    let client = Client::new().with_signer(signer);

    let process_id = "6wqH8ue2-bnJG7j--FV0KGYzSs53ObFDofDITb7qtxI";

    // 1. Add a product
    let product = json!({
        "product_type": "Website DApp",
        "category": "Infrastructure",
        "name": "Aostore",
        "description": "Aostore serves as the Playstore...",
        "website_url": "https://aostore-orpin.vercel.app/",
        "logo_url": "https://pbs.twimg.com/profile_images/...",
        "blockchain": "Arweave",
        "referral_fee": 0.02
    }).to_string();

    let tags = vec![Tag::new("Action", "AddProduct")];
    let msg_id = client.send_message(process_id, product.as_bytes(), tags, None).await?;
    let result = client.wait_for_result(&msg_id, process_id, 30, 2).await?;
    println!("AddProduct result: {}", String::from_utf8_lossy(&result.output));

    // 2. Fetch all apps
    let tags = vec![Tag::new("Action", "FetchAllApps")];
    let msg_id = client.send_message(process_id, &[], tags, None).await?;
    let result = client.wait_for_result(&msg_id, process_id, 30, 2).await?;
    let apps: serde_json::Value = serde_json::from_slice(&result.output)?;
    println!("Total apps: {}", apps.as_array().unwrap().len());

    Ok(())
}

# 🧑‍💻 Contributing
Contributions are welcome! Please open an issue or pull request. Make sure to:

Run cargo fmt to format code.

Run cargo test to ensure tests pass.

Add tests for new functionality.

# License
This project is licensed under the MIT License. See the LICENSE file for details.

# 🙏 Acknowledgements
Built on top of arweave-rs for Arweave wallet operations.
