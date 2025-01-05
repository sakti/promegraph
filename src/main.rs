use anyhow::Result;
mod promql;

use promql::get_data;

#[tokio::main]
async fn main() -> Result<()> {
    println!("promegraph");

    println!("{}", get_data("up").await.unwrap());

    Ok(())
}
