use core_modules::chat_completions::interfaces::chat::start_chat;
use tokio::main;

#[main]
async fn main() {
    // Start the chat interface
    start_chat().await;
}
