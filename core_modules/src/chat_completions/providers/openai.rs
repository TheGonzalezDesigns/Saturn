pub mod openai {
    use anyhow::{bail, Error, Result};
    use dotenv::dotenv;
    use reqwest::{header, Client, StatusCode};
    use serde::{Deserialize, Serialize};
    use std::env;

    #[derive(Serialize)]
    pub struct OpenAIPayloadMessage {
        role: String,
        content: String,
    }

    impl OpenAIPayloadMessage {
        pub fn new(role: String, content: String) -> Self {
            OpenAIPayloadMessage { role, content }
        }
    }

    pub type OpenAIPayloadMessages = Vec<OpenAIPayloadMessage>;

    #[derive(Serialize)]
    pub struct OpenAIPayload {
        model: String,
        messages: OpenAIPayloadMessages,
    }

    impl OpenAIPayload {
        pub fn new(model: String, query: String) -> Self {
            let system_message = OpenAIPayloadMessage::new(
                "system".to_string(),
                "You are a helpful assistant.".to_string(),
            );
            let user_message = OpenAIPayloadMessage::new("user".to_string(), query);
            let messages = vec![system_message, user_message];
            OpenAIPayload { model, messages }
        }
    }

    #[derive(Deserialize)]
    pub struct OpenAIChatCompletionResponseChoiceMessage {
        content: String,
    }

    #[derive(Deserialize)]
    pub struct OpenAIChatCompletionResponseChoice {
        message: OpenAIChatCompletionResponseChoiceMessage,
    }

    pub type OpenAIChatCompletionResponseChoices = Vec<OpenAIChatCompletionResponseChoice>;

    #[derive(Deserialize)]
    pub struct OpenAIChatCompletionResponse {
        choices: OpenAIChatCompletionResponseChoices,
    }

    pub async fn openai(query: String) -> Result<String, Error> {
        dotenv().ok();
        let openai_api_key = env::var("OPENAI_API_KEY").expect("Failed to extract OPENAI_API_KEY");
        let client = Client::new();
        let payload = OpenAIPayload::new("gpt-4o".to_string(), query);

        let response = client
            .post("https://api.openai.com/v1/chat/completions")
            .header(header::CONTENT_TYPE, "application/json; charset=utf-8")
            .header(header::AUTHORIZATION, format!("Bearer {}", openai_api_key))
            .json(&payload)
            .send()
            .await?;

        if response.status() != StatusCode::OK {
            bail!("Failed to fetch response: {}", response.status());
        }

        let completion: OpenAIChatCompletionResponse = response.json().await?;

        // Extract the content from the first choice
        if let Some(choice) = completion.choices.first() {
            Ok(choice.message.content.clone())
        } else {
            bail!("No choices found in the response.")
        }
    }
}

pub mod openai_json {
    use anyhow::{bail, Error, Result};
    use dotenv::dotenv;
    use reqwest::{header, Client, StatusCode};
    use serde::{Deserialize, Serialize};
    use serde_json::{json, Value};
    use std::env;

    #[derive(Serialize, Debug)]
    pub struct OpenAIPayloadMessage {
        role: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,
        content: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        function_call: Option<Value>,
    }

    impl OpenAIPayloadMessage {
        pub fn new(
            role: String,
            content: Option<String>,
            function_call: Option<Value>,
            name: Option<String>,
        ) -> Self {
            OpenAIPayloadMessage {
                role,
                content,
                function_call,
                name,
            }
        }
    }

    pub type OpenAIPayloadMessages = Vec<OpenAIPayloadMessage>;

    #[derive(Serialize, Debug)]
    pub struct OpenAIFunctionParameter {
        #[serde(rename = "type")]
        param_type: String,
        properties: Value,
        required: Vec<String>,
    }

    #[derive(Serialize, Debug)]
    pub struct OpenAIFunctionDetails {
        name: String,
        description: String,
        parameters: OpenAIFunctionParameter,
    }

    #[derive(Serialize, Debug)]
    pub struct OpenAIFunction {
        r#type: String,
        function: OpenAIFunctionDetails,
    }

    #[derive(Serialize, Debug)]
    pub struct OpenAIFunctionPayload {
        model: String,
        messages: OpenAIPayloadMessages,
        tools: Vec<OpenAIFunction>,
        tool_choice: String,
    }

    impl OpenAIFunctionPayload {
        pub fn new(
            model: String,
            user_query: String,
            function_name: String,
            function_description: String,
            mut properties: Value,
            mut required: Vec<String>,
            function_call_arguments: Value,
        ) -> Self {
            // Define the is_valid_json_response property
            let is_valid_json_property = json!({
                "is_valid_json_response": {
                    "type": "boolean",
                    "description": "Indicates if the response is valid JSON."
                }
            });

            // Merge with the user-defined properties
            if let Some(props) = properties.as_object_mut() {
                props.extend(is_valid_json_property.as_object().unwrap().clone());
            }

            // Ensure `is_valid_json_response` is in the required fields
            if !required.contains(&"is_valid_json_response".to_string()) {
                required.push("is_valid_json_response".to_string());
            }

            let user_message = OpenAIPayloadMessage::new(
                "user".to_string(),
                Some(format!(
                    "Respond to the following query in perfect json: {user_query}"
                )),
                None,
                None,
            );

            let assistant_message = OpenAIPayloadMessage::new(
                "assistant".to_string(),
                None,
                Some(
                    json!({ "name": function_name, "arguments": serde_json::to_string(&function_call_arguments).unwrap() }),
                ),
                None,
            );

            let function_message = OpenAIPayloadMessage::new(
                "function".to_string(),
                Some(serde_json::to_string(&function_call_arguments).unwrap()),
                None,
                Some(function_name.clone()),
            );

            let messages = vec![user_message, assistant_message, function_message];

            let parameters = OpenAIFunctionParameter {
                param_type: "object".to_string(),
                properties,
                required,
            };

            let function_details = OpenAIFunctionDetails {
                name: function_name.clone(),
                description: function_description,
                parameters,
            };

            let tool = OpenAIFunction {
                r#type: "function".to_string(),
                function: function_details,
            };

            OpenAIFunctionPayload {
                model,
                messages,
                tools: vec![tool],
                tool_choice: "auto".to_string(),
            }
        }
    }

    #[derive(Deserialize, Debug)]
    pub struct OpenAIChatCompletionResponseChoiceMessageToolCall {
        id: String,
        r#type: String,
        function: FunctionCallDetails,
    }

    #[derive(Deserialize, Debug)]
    pub struct FunctionCallDetails {
        name: String,
        arguments: String,
    }

    #[derive(Deserialize, Debug)]
    pub struct OpenAIChatCompletionResponseChoiceMessage {
        content: Option<String>,
        tool_calls: Option<Vec<OpenAIChatCompletionResponseChoiceMessageToolCall>>,
    }

    #[derive(Deserialize, Debug)]
    pub struct OpenAIChatCompletionResponseChoice {
        message: OpenAIChatCompletionResponseChoiceMessage,
    }

    pub type OpenAIChatCompletionResponseChoices = Vec<OpenAIChatCompletionResponseChoice>;

    #[derive(Deserialize, Debug)]
    pub struct OpenAIChatCompletionResponse {
        pub choices: OpenAIChatCompletionResponseChoices,
    }

    // Flexible function call method
    pub async fn function_call(
        query: String,
        function_name: String,
        function_description: String,
        properties: Value,
        required: Vec<String>,
        function_call_arguments: Value,
    ) -> Result<Value, Error> {
        dotenv().ok();
        let openai_api_key = env::var("OPENAI_API_KEY").expect("Failed to extract OPENAI_API_KEY");
        let client = Client::new();

        let payload = OpenAIFunctionPayload::new(
            "gpt-4o".to_string(),
            query,
            function_name,
            function_description,
            properties,
            required,
            function_call_arguments,
        );

        let response = client
            .post("https://api.openai.com/v1/chat/completions")
            .header(header::CONTENT_TYPE, "application/json; charset=utf-8")
            .header(header::AUTHORIZATION, format!("Bearer {}", openai_api_key))
            .json(&payload)
            .send()
            .await?;

        if response.status() != StatusCode::OK {
            bail!("Failed to fetch response: {}", response.status());
        }

        let completion: OpenAIChatCompletionResponse = response.json().await?;

        println!("completion: {:?}", completion);

        if let Some(choice) = completion.choices.first() {
            if let Some(tool_calls) = &choice.message.tool_calls {
                if let Some(tool_call) = tool_calls.first() {
                    let arguments: Value = serde_json::from_str(&tool_call.function.arguments)?;
                    return Ok(arguments);
                }
            }

            if let Some(content) = &choice.message.content {
                let content_json: Value = serde_json::from_str(content)?;
                return Ok(content_json);
            }
        }

        bail!("No valid response or tool call found in the response.")
    }
}

#[cfg(test)]
mod tests {
    use super::openai_json::*;
    use dotenv::dotenv;
    use serde_json::json;
    use std::env;

    fn setup_openai_key() {
        dotenv().ok();
        if env::var("OPENAI_API_KEY").is_err() {
            panic!("OPENAI_API_KEY is not set in the environment");
        }
    }

    #[tokio::test]
    async fn test_openai_function_call_with_election_data() {
        setup_openai_key();

        // Define properties for election data with the expected response structure
        let properties = json!({
            "candidate": {
                "type": "string",
                "description": "The name of the winning candidate"
            },
            "year": {
                "type": "integer",
                "description": "The year of the election"
            },
            "votes_percentage": {
                "type": "integer",
                "description": "Percentage of popular votes won by the candidate"
            }
        });
        let required = vec!["candidate".to_string(), "year".to_string(), "votes_percentage".to_string()];

        // Define the function arguments
        let function_call_arguments = json!({
            "election_year": 2020
        });

        // Query to determine the winner of the election
        let query = "Who won the 2020 USA presidential election?".to_string();
        let function_name = "get_election_winner".to_string();
        let function_description = "Get the winner of the specified USA presidential election.".to_string();

        match function_call(
            query,
            function_name,
            function_description,
            properties,
            required,
            function_call_arguments,
        )
        .await
        {
            Ok(response) => {
                assert!(
                    response.get("candidate").is_some(),
                    "Expected a candidate field in the response"
                );
                println!("Election Data Response: {:?}", response);
            }
            Err(e) => panic!(
                "Failed to get a response from OpenAI function call: {:?}",
                e
            ),
        }
    }

    #[tokio::test]
    async fn test_openai_function_call_with_location_data_flat() {
        setup_openai_key();

        // Define properties for geographic data
        let properties = json!({
            "latitude": {
                "type": "number",
                "description": "Latitude of the specified location"
            },
            "longitude": {
                "type": "number",
                "description": "Longitude of the specified location"
            }
        });
        let required = vec!["latitude".to_string(), "longitude".to_string()];

        // Define the function arguments
        let function_call_arguments = json!({
            "city": "Havana",
            "country": "Cuba"
        });

        // Query to retrieve coordinates of Havana, Cuba
        let query = "What are the coordinates of Havana, Cuba?".to_string();
        let function_name = "get_location_coordinates".to_string();
        let function_description = "Retrieve latitude and longitude for a specific location.".to_string();

        match function_call(
            query,
            function_name,
            function_description,
            properties,
            required,
            function_call_arguments,
        )
        .await
        {
            Ok(response) => {
                assert!(
                    response.get("latitude").is_some() && response.get("longitude").is_some(),
                    "Expected latitude and longitude fields in the response"
                );
                println!("Location Data Response: {:?}", response);
            }
            Err(e) => panic!(
                "Failed to get a response from OpenAI function call: {:?}",
                e
            ),
        }
    }
}
