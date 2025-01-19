use clap::{Parser, ValueEnum};
use miette::Result;

mod promql;

use promql::get_data;
use textplots::{Chart, Plot, Shape};

#[derive(ValueEnum, Clone, Debug)]
enum Backend {
    Plotters,
    Textplots,
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Promql expression
    #[arg(short, long)]
    expr: String,

    /// Prometheus server address
    #[arg(short, long, default_value = "http://localhost:8428/")]
    addr: String,

    #[arg(short,value_enum, default_value_t = Backend::Textplots)]
    backend: Backend,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    dbg!(&args);
    println!("promegraph");
    match args.backend {
        Backend::Plotters => {
            println!("using plotters");
            println!("{}", get_data(&args.addr, "up").await?);
        }
        Backend::Textplots => {
            println!("using textplots");
            Chart::default()
                .lineplot(&Shape::Continuous(Box::new(|x| x.sin() / x)))
                .display();

            println!("{}", get_data(&args.addr, "up").await?);
            println!();
        }
    }

    Ok(())
}
