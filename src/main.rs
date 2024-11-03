use core_modules::jarvis::jarvis::jarvis;
use core_modules::search::search::search;
use core_modules::gemini::gemini::gemini;
use tokio::main;

#[main]
async fn main() {
    let query: String = String::from("Who's in the last presidential race");
    let search_query: String = String::from("Who's winning the presidential race rn");
    let response = jarvis(query.clone()).await.expect("Failed to call jarvis");
    println!("Jarvis: {query}: {response}");
    let response = gemini(query.clone()).await.expect("Failed to call gemini");
    println!("Gemini: {query}: {response}");
    let search_response = search(search_query.clone()).await.expect("Failed to call search");
    println!("Perplixity: {search_query}: {search_response}");
}
