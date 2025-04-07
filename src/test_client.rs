use std::io::{stdin, stdout, BufRead, BufReader, Write};
use std::net::TcpStream;
use std::process;

fn main() {    
    println!("Enter message to send (Ctrl+D or Ctrl+Z to exit):");
    
    let stream = match TcpStream::connect("127.0.0.1:8080") {
        Ok(s) => {
            println!("Successfully connected to 127.0.0.1:8080");
            s
        }
        Err(e) => {
            eprintln!("Failed to connect: {}", e);
            process::exit(1);
        }
    };

    let mut reader = BufReader::new(stream.try_clone().expect("Failed to clone stream for reading"));
    let mut writer = stream;
    
    loop {
        print!("> ");
        stdout().flush().unwrap();

        let mut input_line = String::new();

        match stdin().read_line(&mut input_line) {
            Ok(n) => {
                if n == 0 {
                    println!("\nInput closed. Exiting.");
                    break;
                }

                if let Err(e) = writer.write_all(input_line.as_bytes()) {
                    eprintln!("Failed to write to stream: {}", e);
                    break;
                }

                if let Err(e) = writer.flush() {
                    eprintln!("Failed to flush stream: {}", e);
                    break;
                }

                let mut response_buffer = String::new();
                match reader.read_line(&mut response_buffer) {
                    Ok(n_resp) => {
                        if n_resp == 0 {
                            println!("\nServer closed the connection.");
                            break;
                        }
                        print!("Server response: {}", response_buffer);
                        stdout().flush().unwrap();
                    }
                    Err(e) => {
                        eprintln!("Failed to read from stream: {}", e);
                        break;
                    }
                }
            }
            Err(error) => {
                eprintln!("Error reading from stdin: {}", error);
                break;
            }
        }
    }
}

