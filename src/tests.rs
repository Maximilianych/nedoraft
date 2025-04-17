mod tests {
    use crate::*;

    #[tokio::test]
    async fn test_state_manager_set_command() {
        let (tx, rx) = mpsc::channel(100);
        let (response_tx, mut response_rx) = mpsc::channel(1);

        tokio::spawn(async move {
            state_manager(rx).await;
        });

        let key = "test_key".to_string();
        let value = "test_value".to_string();
        tx.send(Command::Set { key, value, response_tx }).await.unwrap();

        let response = response_rx.recv().await.unwrap();
        assert_eq!(response, Ok(Some("Ok".to_string())));

        let (response_tx_get, mut response_rx_get) = mpsc::channel(1);
        tx.send(Command::Get { key: "test_key".to_string(), response_tx: response_tx_get }).await.unwrap();
        let get_response = response_rx_get.recv().await.unwrap();
        assert_eq!(get_response, Ok(Some("test_value".to_string())));
    }

    #[tokio::test]
    async fn test_state_manager_get_command() {
        let (tx, rx) = mpsc::channel(100);
        let (response_tx_set, mut response_rx_set) = mpsc::channel(1);
        let (response_tx_get, mut response_rx_get) = mpsc::channel(1);

        tokio::spawn(async move {
            state_manager(rx).await;
        });

        tx.send(Command::Set { key: "test_key".to_string(), value: "test_value".to_string(), response_tx: response_tx_set }).await.unwrap();
        response_rx_set.recv().await.unwrap().unwrap();

        tx.send(Command::Get { key: "test_key".to_string(), response_tx: response_tx_get }).await.unwrap();
        let get_response = response_rx_get.recv().await.unwrap();
        assert_eq!(get_response, Ok(Some("test_value".to_string())));

        let (response_tx_get_nonexistent, mut response_rx_get_nonexistent) = mpsc::channel(1);
        tx.send(Command::Get { key: "nonexistent_key".to_string(), response_tx: response_tx_get_nonexistent }).await.unwrap();
        let get_response_nonexistent = response_rx_get_nonexistent.recv().await.unwrap();
        assert_eq!(get_response_nonexistent, Ok(None));
    }

    #[tokio::test]
    async fn test_state_manager_delete_command() {
        let (tx, rx) = mpsc::channel(100);
        let (response_tx_set, mut response_rx_set) = mpsc::channel(1);
        let (response_tx_delete, mut response_rx_delete) = mpsc::channel(1);
        let (response_tx_get, mut response_rx_get) = mpsc::channel(1);

        tokio::spawn(async move {
            state_manager(rx).await;
        });

        tx.send(Command::Set { key: "delete_key".to_string(), value: "delete_value".to_string(), response_tx: response_tx_set }).await.unwrap();
        response_rx_set.recv().await.unwrap().unwrap();

        tx.send(Command::Detete { key: "delete_key".to_string(), response_tx: response_tx_delete }).await.unwrap();
        let delete_response = response_rx_delete.recv().await.unwrap();
        assert_eq!(delete_response, Ok(Some("delete_value".to_string())));

        tx.send(Command::Get { key: "delete_key".to_string(), response_tx: response_tx_get }).await.unwrap();
        let get_response = response_rx_get.recv().await.unwrap();
        assert_eq!(get_response, Ok(None));
    }

    #[tokio::test]
    async fn test_state_manager_error_command() {
        let (tx, rx) = mpsc::channel(100);
        let (response_tx, mut response_rx) = mpsc::channel(1);

        tokio::spawn(async move {
            state_manager(rx).await;
        });

        tx.send(Command::Error { response_tx }).await.unwrap();
        let response = response_rx.recv().await.unwrap();
        assert_eq!(response, Err("Error command".to_string()));
    }
}