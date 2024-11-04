pub mod needs_internet {
    use crate::chat_completions::providers::openai::openai::openai;
    use crate::chat_completions::utils::json_query::json_query::json_query;
    use anyhow::Result;
    use serde_json::{json, Value};

    /// Determines if a given query requires internet access.
    ///
    /// This function calls OpenAI with a provided query and checks if the response indicates
    /// a need for internet access by analyzing the result.
    ///
    /// # Arguments
    /// * `query` - A `String` representing the user query.
    ///
    /// # Returns
    /// * `Result<bool>` - Returns `true` if internet access is required, `false` otherwise.
    pub async fn needs_internet(query: String) -> Result<bool> {
        // First, get the response from OpenAI for the query
        let response = openai(query.clone()).await?;

        // Use json_query to check if the response suggests internet access is needed
        let json_response = json_query(
            "Does this query need internet access".to_string(),
            "check_internet_access".to_string(),
            "Determines if the query's response requires internet access".to_string(),
            json!({
                "needs_internet": {
                    "type": "boolean",
                    "description": "Indicates whether the query requires internet access"
                }
            }),
            vec!["needs_internet".to_string()],
            json!({
                "query": &query,
                "response": &response
            }),
        )
        .await?;

        // Retrieve the "needs_internet" boolean and return it
        Ok(json_response
            .get("needs_internet")
            .and_then(|v| v.as_bool())
            .unwrap_or(false))
    }
}

#[cfg(test)]
mod tests {
    use super::needs_internet::needs_internet;

    #[tokio::test]
    async fn test_needs_internet_for_query() {
        let query: String = "What's the weather like in Orange County, CA?".to_string();
        let requires_internet = needs_internet(query)
            .await
            .expect("Failed to check internet requirement");

        println!("needs_internet: {}", requires_internet);
        assert!(requires_internet, "Expected 'needs_internet' to be true");
    }
}
