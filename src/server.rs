use serde_json::json;
use warp::{http::StatusCode, reply, serve, Filter, Rejection, Reply};
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Server initiating...");
    let query = warp::path("query")
        .and(warp::get())
        .and(warp::body::json())
        .and_then(handle_query);
    println!("Routes registered...");
    println!("Saturn online.");
    serve(query).run(([127, 0, 0, 1], 2223)).await;
    Ok(())
}
async fn handle_query(query: serde_json::Value) -> Result<impl Reply, Rejection> {
    let json_response = json!({ "online": true, "query": query });
    let reply = reply::with_status(json_response.to_string(), StatusCode::OK);
    return Ok(reply);
}
