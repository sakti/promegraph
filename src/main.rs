use clap::{Parser, ValueEnum};
use miette::Result;

mod backend_ratatui;
mod backend_textplots;
mod promql;

use promql::get_data;

#[derive(ValueEnum, Clone, Debug)]
enum Backend {
    Plotters,
    Textplots,
    Ratatui,
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Promql expression
    expr: String,

    /// Prometheus server address
    #[arg(short, long, default_value = "http://localhost:8428/")]
    addr: String,

    /// Step
    #[arg(short, long, default_value_t = 15.0)]
    step: f64,

    /// Duration in minutes
    #[arg(short, long, default_value_t = 1)]
    duration: u16,

    #[arg(short,value_enum, default_value_t = Backend::Textplots)]
    backend: Backend,

    /// Refresh interval in seconds (ratatui backend only)
    #[arg(short, long, default_value_t = 30)]
    refresh: u64,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    match args.backend {
        Backend::Plotters => {
            println!("using plotters");
            get_data(&args.addr, &args.expr, args.step, args.duration).await?;
        }
        Backend::Textplots => {
            let data = get_data(&args.addr, &args.expr, args.step, args.duration).await?;
            let backend = backend_textplots::BackendTextplots::new(200, 60);
            let result = backend.generate(data)?;
            println!("{}", result);
        }
        Backend::Ratatui => {
            let backend = backend_ratatui::BackendRatatui::new(
                args.addr,
                args.expr,
                args.step,
                args.duration,
                args.refresh,
            );
            backend.run().await?;
        }
    }

    Ok(())
}
