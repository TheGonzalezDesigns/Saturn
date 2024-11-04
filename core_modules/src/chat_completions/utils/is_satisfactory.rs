pub mod is_satisfactory {
    use crate::chat_completions::providers::openai::openai::openai;
    use crate::chat_completions::utils::json_query::json_query::json_query;
    use anyhow::Result;
    use serde_json::{json, Value};

    /// Determines if a given response satisfactorily addresses the query.
    ///
    /// This function calls OpenAI with a query and response and checks if the response is satisfactory
    /// by analyzing if it directly and accurately answers the user's question.
    ///
    /// # Arguments
    /// * `query` - A `String` representing the user query.
    /// * `response` - A `String` representing the response to check.
    ///
    /// # Returns
    /// * `Result<bool>` - Returns `true` if the response is satisfactory, `false` otherwise.
    pub async fn is_satisfactory(query: String, response: String) -> Result<bool> {
        // Use json_query to check if the response is satisfactory
        let json_response = json_query(
            "Does this response satisfactorily answer the question".to_string(),
            "check_satisfactory_response".to_string(),
            "Evaluates if the response directly addresses the question with a clear and meaningful answer, avoiding generic or vague language. For example, avoid responses like 'I don't know' or 'Please check yourself'.".to_string(),
            json!({
                "satisfactory": {
                    "type": "boolean",
                    "description": "Indicates whether the response satisfactorily answers the query, providing a direct and useful answer"
                }
            }),
            vec!["satisfactory".to_string()],
            json!({
                "query": &query,
                "response": &response
            }),
        )
        .await?;

        // Retrieve the "satisfactory" boolean and return it
        Ok(json_response
            .get("satisfactory")
            .and_then(|v| v.as_bool())
            .unwrap_or(false))
    }
}

#[cfg(test)]
mod tests {
    use super::is_satisfactory::is_satisfactory;

    #[tokio::test]
    async fn test_is_satisfactory_for_response() {
        let query = "What is the population of New York City?".to_string();
        let response =
            "New York City has a population of approximately 8.4 million people.".to_string();

        let satisfactory = is_satisfactory(query.clone(), response.clone())
            .await
            .expect("Failed to check satisfactory response");

        println!("is_satisfactory: {}", satisfactory);
        assert!(satisfactory, "Expected 'satisfactory' to be true");
    }

    #[tokio::test]
    async fn test_is_unsatisfactory_for_response() {
        let query = "What is the population of New York City?".to_string();
        let response = "I'm sorry, I can't provide real-time data.".to_string();

        let satisfactory = is_satisfactory(query.clone(), response.clone())
            .await
            .expect("Failed to check satisfactory response");

        println!("is_satisfactory: {}", satisfactory);
        assert!(!satisfactory, "Expected 'satisfactory' to be false");
    }

    #[tokio::test]
    async fn test_is_unsatisfactory_for_vague_response() {
        let query = "What's the weather like today?".to_string();
        let response = "I'm not sure, please check online.".to_string();

        let satisfactory = is_satisfactory(query.clone(), response.clone())
            .await
            .expect("Failed to check satisfactory response");

        println!("is_satisfactory: {}", satisfactory);
        assert!(!satisfactory, "Expected 'satisfactory' to be false");
    }
}
