use anyhow::{bail, Result};
use serde_json::{json, Value};
use tokio::time::{sleep, Duration};

use crate::chat_completions::providers::openai::openai_json::function_call;

/// The main function for handling JSON queries with validation and retries.
pub async fn json_query(
    query: String,
    function_name: String,
    function_description: String,
    properties: Value,
    required: Vec<String>,
    function_call_arguments: Value,
) -> Result<Value> {
    // Retry up to 10 times if response does not contain required keys
    for attempt in 1..=10 {
        println!("Attempt: #{attempt}");
        
        // Call the underlying function
        let response = function_call(
            query.clone(),
            function_name.clone(),
            function_description.clone(),
            properties.clone(),
            required.clone(),
            function_call_arguments.clone(),
        )
        .await?;

        println!("Response: {:?}", response);

        // Parse response as JSON
        let response_json: Value = response;

        // Check if all required keys are present
        if has_required_keys(&response_json, &required) {
            return Ok(response_json);
        }

        // Log retry attempt
        eprintln!("Attempt {}/10: Missing required keys, retrying...", attempt);

        // Delay between retries (optional, e.g., 500ms)
        sleep(Duration::from_millis(500)).await;
    }

    // If all attempts fail, return an error
    bail!("Failed to retrieve a response with all required keys after 10 attempts.")
}

/// Check if all required keys are present in the response.
fn has_required_keys(response_json: &Value, required_keys: &[String]) -> bool {
    required_keys.iter().all(|key| response_json.get(key).is_some())
}

#[cfg(test)]
mod tests {
    use super::json_query;
    use super::super::super::providers::openai::openai::openai;
    use serde_json::json;

    #[tokio::test]
    async fn weather_test() {
        let query: String = "What's the weather like in orange county, CA?".to_string();
        let response = openai(query.clone()).await.expect("Weather test error:");
        let json_response = json_query("Does this query need internet access".to_string(),
                      "check_internet_access".to_string(),
                      "Checks if a given query's response is lacking internet access".to_string(),
                      json!({
                        "needs_internet": {
                            "type": "boolean",
                            "description": "true if the text indicates a need to access the internet, false otherwise",
                        }
                      }), vec!["needs_internet".to_string()],
                      json!({
                          "query": &query,
                          "response": &response
                      })).await.expect("OpenAI JSON Response Error:");
        panic!("response: {}", json_response);
    }
}
