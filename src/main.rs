use core_modules::gemini::gemini::gemini;
use core_modules::openai::openai::openai;
use core_modules::openai::openai_json::function_call;
use core_modules::perplexity::perplexity::perplexity;
use serde_json::json;
use tokio::main;

#[main]
async fn main() {
    let query = String::from("What is the weather like today?");
    let function_call_query = String::from("What is the weather like in Boston today?");
    let perplexity_query = String::from("What's the weather forecast for this week?");

    // Call OpenAI standard completion
    let response = openai(query.clone()).await.expect("Failed to call openai");
    println!("Jarvis: {query}: {response}");

    // Define dynamic properties and required fields for the function call related to weather data
    let properties = json!({
        "location": {
            "type": "string",
            "description": "The city and state, e.g., Boston, MA"
        },
        "unit": {
            "type": "string",
            "enum": ["celsius", "fahrenheit"],
            "description": "The temperature unit"
        }
    });
    let required = vec!["location".to_string()];

    // Define arguments for the function call related to weather data
    let function_call_arguments = json!({
        "location": "Boston, MA",
        "unit": "fahrenheit"
    });

    // Call OpenAI function call to get weather information for Boston
    let function_response = function_call(
        function_call_query.clone(),
        "get_current_weather".to_string(),
        "Retrieve the current weather for a given location.".to_string(),
        properties,
        required,
        function_call_arguments,
    )
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
