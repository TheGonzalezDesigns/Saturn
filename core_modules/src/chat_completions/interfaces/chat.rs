use std::io::{self, Write};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::mpsc;
use tokio::task;
use crate::chat_completions::bots::saturn::saturn::saturn;

/// Starts the chat interface with Saturn bot.
pub async fn start_chat() {
    println!("Starting new conversation with Saturn bot.");
    println!("Type 'exit' to end the conversation.\n");

    let (tx, mut rx) = mpsc::channel(1);

    // Spawn a task to read user input asynchronously
    let tx_clone = tx.clone();
    task::spawn(async move {
        let stdin = tokio::io::stdin(); // Use Tokio's async stdin
        let mut reader = BufReader::new(stdin).lines();
        loop {
            print!("You: ");
            io::stdout().flush().unwrap();

            match reader.next_line().await {
                Ok(Some(line)) => {
                    if line.trim().to_lowercase() == "exit" {
                        break;
                    }
                    let _ = tx_clone.send(line).await;
                }
                Ok(None) => break, // End of input
                Err(e) => {
                    eprintln!("Error reading line: {:?}", e);
                    break;
                }
            }
        }
    });

    // Process the chat conversation
    while let Some(query) = rx.recv().await {
        // Send the query to Saturn bot
        match saturn(query.clone()).await {
            Ok(response) => {
                println!("Saturn: {}", response);
            }
            Err(e) => {
                eprintln!("Saturn encountered an error: {:?}", e);
            }
        }
    }

    println!("Conversation ended.");
}
