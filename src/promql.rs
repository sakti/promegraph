use jiff::Timestamp;
use miette::{IntoDiagnostic, Result, miette};
use prometheus_http_query::{Client, response::RangeVector};

pub async fn get_data(
    addr: &str,
    expr: &str,
    step: f64,
    duration: u16,
) -> Result<Vec<RangeVector>> {
    let client = Client::try_from(addr).into_diagnostic()?;
    let start = Timestamp::now().as_second() - (duration as i64 * 60);
    let end = Timestamp::now().as_second();
    let response = client
        .query_range(expr, start, end, step)
        .get()
        .await
        .into_diagnostic()?;
    let result_found = response.data().as_matrix().is_some();
    if !result_found {
        return Err(miette!("range vector not found"));
    }
    let result = response.data().as_matrix().unwrap();
    if result.is_empty() {
        return Err(miette!("empty result"));
    }

    Ok(result.to_vec())
}
