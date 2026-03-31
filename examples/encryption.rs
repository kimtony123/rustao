use rustao::{Client, ARSigner, SendMessageOptions, Tag};
use rsa::pkcs1::EncodeRsaPublicKey;  // <-- add this import
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load your wallet
    let signer = ARSigner::from_file("testKey.json")?;
    let client = Client::new(signer);

    // Generate a dummy recipient key (for demonstration)
    let mut rng = rand::thread_rng();
    let recipient_priv = rsa::RsaPrivateKey::new(&mut rng, 2048)?;
    let recipient_pub = recipient_priv.to_public_key();
    let pub_bytes = recipient_pub.to_pkcs1_der()?.as_ref().to_vec();  // now works

    // Process ID
    let process_id = "6wqH8ue2-bnJG7j--FV0KGYzSs53ObFDofDITb7qtxI";

    // Secret data
    let secret = b"This is a secret message";

    // Send encrypted message
    let options = SendMessageOptions {
        encrypt_with_rsa: Some(pub_bytes),
    };
    let tags = vec![Tag::new("Action", "EncryptedMessage")];

    match client.send_message(process_id, secret, tags, None, Some(options)).await {
        Ok(msg_id) => println!("Encrypted message sent: {}", msg_id),
        Err(e) => eprintln!("Send failed: {}", e),
    }

    Ok(())
}