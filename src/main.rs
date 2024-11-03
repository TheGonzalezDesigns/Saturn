use core_modules::gemini::gemini::gemini;
use core_modules::openai::openai::openai;
use core_modules::perplexity::perplexity::perplexity;
use tokio::main;

#[main]
async fn main() {
    let query: String = String::from("Who's in the last presidential race");
    let perplexity_query: String = String::from("Who's winning the presidential race rn");
    let response = openai(query.clone()).await.expect("Failed to call openai");
    println!("Jarvis: {query}: {response}");
    let response = gemini(query.clone()).await.expect("Failed to call gemini");
    println!("Gemini: {query}: {response}");
    let perplexity_response = perplexity(perplexity_query.clone())
        .await
        .expect("Failed to call perplexity");
    println!("Perplixity: {perplexity_query}: {perplexity_response}");
}
