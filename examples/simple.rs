use rustao::{Client, ARSigner, Tag};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load your Arweave wallet (JWK file)
    let signer = ARSigner::from_file("testKey.json")?;

    // Create a client (uses default gateways)
    let client = Client::new(signer);

    // Process ID that returns 42 on /counter (if reachable)
    let process_id = "6wqH8ue2-bnJG7j--FV0KGYzSs53ObFDofDITb7qtxI";

    // Query compute
    match client.get_compute_string(process_id, "counter").await {
        Ok(counter) => println!("Counter: {}", counter),
        Err(e) => eprintln!("Compute failed: {}", e),
    }

    // Send a message
    let tags = vec![Tag::new("Action", "Message")];
    match client.send_message(process_id, b"Hello AO!", tags, None, None).await {
        Ok(msg_id) => println!("Message sent: {}", msg_id),
        Err(e) => eprintln!("Send failed: {}", e),
    }

    Ok(())
}