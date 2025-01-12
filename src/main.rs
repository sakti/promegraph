use anyhow::Result;
use clap::Parser;

mod promql;

use promql::get_data;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Promql expression
    #[arg(short, long)]
    expr: String,

    /// Prometheus server address
    #[arg(short, long)]
    addr: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    dbg!(args);
    println!("promegraph");

    println!("{}", get_data("up").await.unwrap());

    Ok(())
}
