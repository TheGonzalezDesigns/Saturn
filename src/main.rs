use core_modules::jarvis::jarvis::jarvis;
use core_modules::search::search::search;
use tokio::main;

#[main]
async fn main() {
    let query: String = String::from("Who's in the last presidential race");
    let search_query: String = String::from("Who's winning the presidential race rn");
    let response = jarvis(query.clone()).await.expect("Failed to call jarvis");
    println!("{query}: {response}");
    let search_response = search(search_query.clone()).await.expect("Failed to call search");
    println!("{search_query}: {search_response}");
}
