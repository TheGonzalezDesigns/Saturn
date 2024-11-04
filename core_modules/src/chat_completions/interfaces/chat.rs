use crate::chat_completions::bots::saturn::saturn::saturn;
use std::io::{self, Write};
use tokio::io::stdin as async_stdin;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::time::{sleep, Duration};

const USER_COLOR: (u8, u8, u8) = (23, 184, 144); // Cyan color for user
const AI_COLOR: (u8, u8, u8) = (255, 223, 0); // Yellow color for AI
const THOUGHT_COLOR: (u8, u8, u8) = (100, 100, 100); // Gray for "THINKING..."

/// Set the terminal color
fn set_color(r: u8, g: u8, b: u8) {
    print!("\x1b[38;2;{};{};{}m", r, g, b);
}

/// Reset the terminal color
fn reset_color() {
    print!("\x1b[0m");
}

/// Print with a color
fn print_colored(text: &str, color: (u8, u8, u8)) {
    set_color(color.0, color.1, color.2);
    println!("{}", text);
    reset_color();
}

/// Simulates typing effect for AI responses
async fn typing_effect(text: &str, color: (u8, u8, u8)) {
    set_color(color.0, color.1, color.2);
    for c in text.chars() {
        print!("{}", c);
        io::stdout().flush().unwrap();
        sleep(Duration::from_millis(50)).await;
    }
    println!();
    reset_color();
}

/// Starts the chat interface with Saturn bot
pub async fn start_chat() {
    println!("Starting new conversation with Saturn bot.");
    println!("Type 'exit' to end the conversation.\n");

    let stdin = async_stdin();
    let mut reader = BufReader::new(stdin).lines();

    loop {
        print_colored("You:", USER_COLOR);
        print!("Send a message ('exit' to quit): ");
        io::stdout().flush().unwrap();

        if let Ok(Some(input)) = reader.next_line().await {
            if input.trim().eq_ignore_ascii_case("exit") {
                print_colored("Goodbye!", THOUGHT_COLOR);
                break;
            }

            // Display "THINKING..." while AI processes the input
            print_colored("THINKING...", THOUGHT_COLOR);

            // Send the query to Saturn bot and process the response
            match saturn(input.clone()).await {
                Ok(response) => {
                    print_colored("Saturn:", AI_COLOR);
                    typing_effect(&response, AI_COLOR).await;
                }
                Err(e) => {
                    eprintln!("Saturn encountered an error: {:?}", e);
                }
            }
        }
    }

    println!("Conversation ended.");
}
