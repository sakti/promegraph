use backend::Generator;
use clap::{Parser, ValueEnum};
use miette::Result;

mod backend;
mod backend_textplots;
mod promql;

use promql::get_data;

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

    /// Step
    #[arg(short, long, default_value_t = 15.0)]
    step: f64,

    /// Duration in minutes
    #[arg(short, long, default_value_t = 1)]
    duration: u8,

    #[arg(short,value_enum, default_value_t = Backend::Textplots)]
    backend: Backend,
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
            // for (_i, v) in result.iter().enumerate() {
            //     let _metric_name = v.metric().get("__name__").cloned().unwrap_or_default();
            //     let points: Vec<(f64, f64)> = v
            //         .samples()
            //         .into_iter()
            //         .map(|s| (s.timestamp(), s.value()))
            //         .collect();
            //     let (_xmin, _xmax, ymin, ymax) = get_bounds(&points);

            //     let vec32: Vec<(f32, f32)> =
            //         points.iter().map(|(x, y)| (*x as f32, *y as f32)).collect();

            //     println!("{}", args.expr);
            //     Chart::new_with_y_range(
            //         200,
            //         60,
            //         0.0,
            //         points.len() as f32,
            //         (ymin - 0.01) as f32,
            //         (ymax + 0.01) as f32,
            //     )
            //     .lineplot(&Shape::Continuous(Box::new(|x| {
            //         let idx = x as usize;
            //         if idx < vec32.len() {
            //             vec32[idx].1
            //         } else {
            //             0.0
            //         }
            //     })))
            //     .x_label_format(LabelFormat::Custom(Box::new(move |val| {
            //         let idx = val as usize;
            //         if idx < points.len() {
            //             let ts = Timestamp::from_second(points[idx].0 as i64).unwrap();
            //             ts.to_zoned(TimeZone::system()).to_string()
            //             // format!("{}", points[idx].0)
            //         } else {
            //             // get last value
            //             // format!("{}", points[points.len() - 1].0)
            //             let ts = Timestamp::from_second(points[points.len() - 1].0 as i64).unwrap();
            //             ts.to_zoned(TimeZone::system()).to_string()
            //             // format!("{}", val as usize)
            //         }
            //         // format!("{}", points[val as usize].0)
            //         // format!("{}", val as usize)
            //     })))
            //     .y_label_format(LabelFormat::Value)
            //     .y_tick_display(TickDisplay::Sparse)
            //     .display();
            // }
            println!("{}", result);
        }
    }

    Ok(())
}
