use jiff::Timestamp;
use miette::{IntoDiagnostic, Result};
use prometheus_http_query::Client;

pub async fn get_data(addr: &str, expr: &str) -> Result<String> {
    let client = Client::try_from(addr).into_diagnostic()?;
    let start = Timestamp::now().as_second() - (5 * 60);
    let end = Timestamp::now().as_second();
    dbg!(start);
    dbg!(end);
    let response = client
        .query_range(expr, start, end, 60.0)
        .get()
        .await
        .into_diagnostic()?;
    dbg!(response);
    Ok("the data".to_string())
}
