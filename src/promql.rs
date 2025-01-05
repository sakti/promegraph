use anyhow::{Ok, Result};
use jiff::Timestamp;
use prometheus_http_query::Client;

pub async fn get_data(expr: &str) -> Result<String> {
    let client = Client::try_from("http://localhost:8428/")?;
    let start = Timestamp::now().as_second() - (5 * 60);
    let end = Timestamp::now().as_second();
    dbg!(start);
    dbg!(end);
    let response = client.query_range(expr, start, end, 60.0).get().await?;
    dbg!(response);
    Ok("the data".to_string())
}
