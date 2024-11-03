use core_modules::gemini::gemini::gemini;
use core_modules::openai::openai::{openai, function_call};
use core_modules::perplexity::perplexity::perplexity;
use tokio::main;

#[main]
async fn main() {
    let query = String::from("Who's in the last presidential race");
    let function_call_query = String::from("What's the weather like in Boston today?");
    let perplexity_query = String::from("Who's winning the presidential race rn");

    // Call OpenAI standard completion
    let response = openai(query.clone()).await.expect("Failed to call openai");
    println!("Jarvis: {query}: {response}");

    // Call OpenAI function call
    let function_response = function_call(function_call_query.clone())
        .await
        .expect("Failed to call OpenAI function");
    println!("OpenAI Function Call: {function_call_query}: {function_response}");

    // Call Gemini
    let gemini_response = gemini(query.clone()).await.expect("Failed to call gemini");
    println!("Gemini: {query}: {gemini_response}");

    // Call Perplexity
    let perplexity_response = perplexity(perplexity_query.clone())
        .await
        .expect("Failed to call perplexity");
    println!("Perplexity: {perplexity_query}: {perplexity_response}");
}
