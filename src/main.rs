use core_modules::chat_completions::bots::saturn::saturn::saturn;
use tokio::main;

#[main]
async fn main() {
    // Define queries for testing different types of responses.
    let historical_query = String::from("Who won the last USA presidential election?");
    let live_update_query = String::from("What is the weather like today in Boston?");
    let forecast_query = String::from("What's the weather forecast for this week in New York City?");

    // Call Saturn bot for a historical question
    let historical_response = saturn(historical_query.clone())
        .await
        .expect("Saturn failed to respond to the historical query");
    println!("Saturn (Historical): {historical_query}: {historical_response}");

    // Call Saturn bot for a live update question (weather today)
    let live_update_response = saturn(live_update_query.clone())
        .await
        .expect("Saturn failed to respond to the live update query");
    println!("Saturn (Live Update): {live_update_query}: {live_update_response}");

    // Call Saturn bot for a weekly forecast, expecting internet access requirement
    let forecast_response = saturn(forecast_query.clone())
        .await
        .expect("Saturn failed to respond to the forecast query");
    println!("Saturn (Forecast): {forecast_query}: {forecast_response}");
}
