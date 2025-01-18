use anyhow::Result;
use clap::{Parser, ValueEnum};

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
    #[arg(short, long)]
    addr: Option<String>,

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
            println!("{}", get_data("up").await.unwrap());
        }
        Backend::Textplots => {
            println!("using textplots");
            Chart::default()
                .lineplot(&Shape::Continuous(Box::new(|x| x.sin() / x)))
                .display();

            println!();
            println!();
            // new chart
            Chart::new(300, 100, -20.0, 20.0)
                .lineplot(&Shape::Continuous(Box::new(|x| x.cos())))
                .lineplot(&Shape::Continuous(Box::new(|x| x.sin() / 2.0)))
                .display();

            println!();
            let points = [
                (-10.0, -1.0),
                (0.0, 0.0),
                (1.0, 1.0),
                (2.0, 0.0),
                (3.0, 3.0),
                (4.0, 4.0),
                (5.0, 3.0),
                (9.0, 1.0),
                (10.0, -1.0),
            ];

            println!("\ny = interpolated points");
            Chart::default().lineplot(&Shape::Lines(&points)).display();

            println!("\ny = staircase points");
            Chart::default().lineplot(&Shape::Steps(&points)).display();

            println!("\ny = scatter plot");
            Chart::default().lineplot(&Shape::Points(&points)).display();
        }
    }

    Ok(())
}
