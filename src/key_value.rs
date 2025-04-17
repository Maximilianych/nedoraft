use std::collections::HashMap;

use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
    sync::mpsc,
};

mod tests;

#[derive(Debug)]
enum Command {
    Set { key: String, value: String, response_tx: mpsc::Sender<Result<Option<String>, String>> },
    Get { key: String, response_tx: mpsc::Sender<Result<Option<String>, String>> },
    Detete { key: String, response_tx: mpsc::Sender<Result<Option<String>, String>> },
    Error { response_tx: mpsc::Sender<Result<Option<String>, String>> }
}

async fn state_manager(mut rx: mpsc::Receiver<Command>) {
    let mut storage: HashMap<String, String> = HashMap::new();

    while let Some(cmd) = rx.recv().await {
        match cmd {
            Command::Set { key, value, response_tx } => {
                storage.insert(key, value);
                let _ = response_tx.send(Ok(Some("Ok".to_string()))).await;
                println!("Set command completed");
            }
            Command::Get { key, response_tx } => {
                let result = storage.get(&key).cloned();
                let _ = response_tx.send(Ok(result)).await;
                println!("Get command completed");
            }
            Command::Detete { key, response_tx } => {
                let result = storage.remove(&key);
                let _ = response_tx.send(Ok(result)).await;
                println!("Delete command completed");
            }
            Command::Error {response_tx} => {
                let _ = response_tx.send(Err("Error command".to_string())).await;
                eprintln!("Invalid command received");
            }
        }
        println!("HashMap: {:?}", storage);
    };
}

async fn handle_connection(mut stream: TcpStream, tx: mpsc::Sender<Command>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    loop {
        let (response_tx, mut response_rx) = mpsc::channel(1);

        let (reader, mut writer) = stream.split();
        let mut buf_reader = BufReader::new(reader);

        let mut raw_string = String::new();
        match buf_reader.read_line(&mut raw_string).await {
            Ok(0) => {
                println!("Client disconnected before sending command.");
                return Ok(());
            }
            Ok(_) => {
                println!("Received raw line: {:?}", raw_string);
            }
            Err(e) => {
                eprintln!("Failed to read line from client: {}", e);
                return Err(e.into());
            }
        }
        // Parser
        println!("Recived string: {}", raw_string.trim());
        let cmd: Vec<_> = raw_string.trim().split_whitespace().collect();
        println!("Splited cmd: {:?}", cmd);

        let command = if !cmd.is_empty() {
            match cmd[0].to_uppercase().as_str() {
                "SET" if cmd.len() >= 3 => {
                    let value = cmd[2..].join(" ");
                    Command::Set {
                        key: cmd[1].to_string(),
                        value,
                        response_tx,
                    }
                }
                "GET" if cmd.len() >= 2 => Command::Get {
                    key: cmd[1].to_string(),
                    response_tx,
                },
                "DELETE" if cmd.len() >= 2 => Command::Detete {
                    key: cmd[1].to_string(),
                    response_tx,
                },
                _ => Command::Error { response_tx },
            }
        } else {
            Command::Error { response_tx }
        };
        println!("Command: {:?}", command);

        if let Err(e) = tx.send(command).await {
            eprintln!("Failed to send command to state_manager task: {}", e);
            return Ok(());
        }

        match response_rx.recv().await {
            Some(Ok(Some(mut response))) => {
                println!("Sending success response: {}", response);
                if !response.ends_with('\n') {
                    response.push('\n');
                }
                if let Err(e) = writer.write_all(response.as_bytes()).await {
                    eprintln!("Failed to write response to client: {}", e);
                }
            }
            Some(Ok(None)) => {
                println!("Sending empty/not-found response");
                if let Err(e) = writer.write_all(b"\n").await {
                    eprintln!("Failed to write empty response to client: {}", e);
                }
            }
            Some(Err(e)) => { 
                println!("Sending error response: {}", e);
                if let Err(write_err) = writer.write_all(e.as_bytes()).await {
                    eprintln!("Failed to write error response to client: {}", write_err);
                }
                if !e.ends_with('\n') {
                    let _ = writer.write_all(b"\n").await;
                }
            }
            None => {
                eprintln!("Response channel closed unexpectedly.");
                let error_msg = "Internal server error: Handler task failed.\n";
                if let Err(e) = writer.write_all(error_msg.as_bytes()).await {
                    eprintln!("Failed to write internal error message to client: {}", e);
                }
            }
        }

        if let Err(e) = writer.flush().await {
            eprintln!("Failed to flush writer: {}", e);
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    let (tx, rx) = mpsc::channel(100);

    tokio::spawn(state_manager(rx));

    println!("Server listening on 127.0.0.1:8080");

    while let Ok((stream, _)) = listener.accept().await {
        println!("New connection: {}", stream.peer_addr().unwrap());
        let tx_clone = tx.clone();
        tokio::spawn(async move {
            if let Err(e) = handle_connection(stream, tx_clone).await {
                eprintln!("Error handling connection: {e}");
            }
        });
    }
    
    Ok(())
}
