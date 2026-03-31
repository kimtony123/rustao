#[cfg(test)]
mod tests {
    use rustao::{Client, ARSigner, Tag};
    use std::env;
    use std::path::PathBuf;
    use std::time::Duration;

    fn integration_enabled() -> bool {
        env::var("RUSTAO_INTEGRATION").is_ok() || env::var("PYAO_INTEGRATION").is_ok()
    }

    fn wallet_path() -> Option<PathBuf> {
        if let Some(path) = env::var_os("RUSTAO_WALLET_PATH") {
            Some(PathBuf::from(path))
        } else if let Some(path) = env::var_os("PYAO_WALLET_PATH") {
            Some(PathBuf::from(path))
        } else {
            let default = PathBuf::from("testKey.json");
            if default.exists() { Some(default) } else { None }
        }
    }

    #[tokio::test]
    async fn test_compute_counter() {
        if !integration_enabled() {
            eprintln!("Skipping integration test; set RUSTAO_INTEGRATION=1 to run");
            return;
        }

        let wallet_path = wallet_path().expect("testKey.json not found");
        let signer = ARSigner::from_file(wallet_path).expect("failed to load wallet");
        let client = Client::new(signer);

        let process_id = "6wqH8ue2-bnJG7j--FV0KGYzSs53ObFDofDITb7qtxI";
        let result = client.get_compute_string(process_id, "counter").await;
        match result {
            Ok(val) => println!("Counter = {}", val),
            Err(e) => eprintln!("Compute failed: {}", e),
        }
    }

    #[tokio::test]
    async fn test_send_and_wait() {
        if !integration_enabled() {
            eprintln!("Skipping integration test; set RUSTAO_INTEGRATION=1 to run");
            return;
        }

        let wallet_path = wallet_path().expect("testKey.json not found");
        let signer = ARSigner::from_file(wallet_path).expect("failed to load wallet");
        let client = Client::new(signer);

        let process_id = "6wqH8ue2-bnJG7j--FV0KGYzSs53ObFDofDITb7qtxI";
        let tags = vec![Tag::new("Action", "Message")];

        // Store the message ID if send succeeds
        let maybe_id = match client.send_message(process_id, b"hello", tags, None, None).await {
            Ok(id) => {
                println!("Message sent: {}", id);
                Some(id)
            }
            Err(e) => {
                eprintln!("Send failed: {}", e);
                None
            }
        };

        if let Some(id) = maybe_id {
            match client.wait_for_result(&id, process_id, Duration::from_secs(30), Duration::from_secs(2)).await {
                Ok(res) => println!("Result: {:?}", res.output),
                Err(e) => eprintln!("Wait failed: {}", e),
            }
        }
    }
}