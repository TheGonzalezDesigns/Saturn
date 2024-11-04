use crate::chat_completions::bots::saturn::saturn::saturn;
use std::io::{self, Write};
use tokio::io::{AsyncBufReadExt, BufReader};

/// Starts the chat interface with Saturn bot.
pub async fn start_chat() {
    println!("Starting new conversation with Saturn bot.");
    println!("Type 'exit' to end the conversation.\n");

    let stdin = tokio::io::stdin();
    let mut reader = BufReader::new(stdin).lines();

    loop {
        // Prompt the user
        print!("You: ");
        io::stdout().flush().unwrap();

        // Read the input line
        match reader.next_line().await {
            Ok(Some(line)) => {
                let query = line.trim().to_string();
                if query.to_lowercase() == "exit" {
                    println!("Conversation ended.");
                    break;
                }

                // Send the query to Saturn bot and get the response
                match saturn(query.clone()).await {
                    Ok(response) => {
                        println!("Saturn: {}", response);
                    }
                    Err(e) => {
                        eprintln!("Saturn encountered an error: {:?}", e);
                    }
                }
            }
            Ok(None) => {
                println!("End of input.");
                break;
            }
            Err(e) => {
                eprintln!("Error reading line: {:?}", e);
                break;
            }
        }
    }
}
