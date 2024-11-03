use core_modules::jarvis::jarvis::jarvis;
use tokio::main;

#[main]
async fn main() {
    let query: String = String::from("Who's winning the presidential race rn");
    let response = jarvis(query).await.expect("Failed to call jarvis");
    println!("Hello, world!: {response}");
}
