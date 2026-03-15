mod query;

use worker::*;

use crate::query::{RatingQuery, query_params};

#[event(fetch)]
async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    let router = Router::new();

    router.get_async("/rating", |req, _ctx| async move {
        let (name, postcode) = query_params::<RatingQuery>(&req)?.validate()?;

        let raw_data = match request_rating(&name, &postcode).await {
            Ok(body) => body,
            Err((msg, status)) => return Response::error(msg, status)
        };

        let parsed = match parse_rating_json(raw_data) {
            Ok(data) => data,
            Err((msg, status)) => return Response::error(msg, status)
        };

        Response::from_json(&parsed)
    }).run(req, env).await
}

async fn request_rating(name: &String, postcode: &String) -> Result<serde_json::Value, (&'static str, u16)>{
    let url = format!("https://api.ratings.food.gov.uk/Establishments?name={}&address={}", name, postcode);

    let headers = Headers::new();
    headers.set("Accept", "application/json")
        .map_err(|_| ("Failed to set header: 'Accept'", 500))?;
    headers.set("x-api-version", "2")
        .map_err(|_| ("Failed to set header: 'x-api-version'", 500))?;

    let mut init = RequestInit::new();
    init.with_method(Method::Get).with_headers(headers);

    let request = Request::new_with_init(&url, &init)
        .map_err(|_| ("Failed to construct request", 500))?;

        let mut response = Fetch::Request(request)
        .send()
        .await
        .map_err(|_| ("Failed to reach FSA API", 502))?;

    let raw_response = match response.status_code() {
        200 => response.text().await.map_err(|_| ("Failed to read response body", 502))?,
        429 => Err(("Rate limited by FSA API", 502))?,
        _ => Err(("FSA API returned an unexpected response", 502))?
    };

    
    serde_json::from_str(&raw_response).map_err(|_| ("Failed to parse FSA response", 502))
}

fn parse_rating_json(json: serde_json::Value) -> Result<serde_json::Value, (&'static str, u16)> {
    let establishments = json["establishments"]
        .as_array()
        .ok_or(("Unexpected FSA response structure", 502))?;

    let results: Vec<serde_json::Value> = establishments.iter().map(|e| {
        serde_json::json!({
            "name": e["BusinessName"],
            "postcode": e["PostCode"],            
            "rating": e["RatingValue"],
            "ratingDate": e["RatingDate"].as_str().and_then(|d| d.split('T').next()).unwrap_or(""),
            "newRatingPending": e["NewRatingPending"]
        })
    }).collect();

    Ok(serde_json::json!({ "results": results }))
}