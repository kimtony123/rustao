#[cfg(test)]
mod tests {
    use rustao::{Client, ARSigner, Tag, SendMessageOptions};

    #[tokio::test]
    async fn test_client_creation() {
        let signer = ARSigner::from_file("testKey.json").expect("test wallet required");
        let client = Client::new(signer);
        assert_eq!(client.mu, "https://mu.ao.arweave.net");
        assert_eq!(client.cu, "https://cu.ao.arweave.net");
        assert_eq!(client.compute_gateway, "https://push.forward.computer");
    }

    #[tokio::test]
    #[ignore = "requires live network and wallet"]
    async fn test_send_message() {
        let signer = ARSigner::from_file("testKey.json").unwrap();
        let client = Client::new(signer);
        let process_id = "6wqH8ue2-bnJG7j--FV0KGYzSs53ObFDofDITb7qtxI";
        let tags = vec![Tag::new("Action", "Test")];
        let result = client.send_message(process_id, b"hello", tags, None, None).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_send_message_options_encryption() {
        let options = SendMessageOptions {
            encrypt_with_rsa: Some(vec![1, 2, 3]),
        };
        assert!(options.encrypt_with_rsa.is_some());
    }
}