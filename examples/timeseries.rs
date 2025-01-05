use chrono::{TimeZone, Utc};
use plotters::prelude::*;

const DATA: [f64; 14] = [
    137.24, 136.37, 138.43, 137.41, 139.69, 140.41, 141.58, 139.55, 139.68, 139.10, 138.24, 135.67,
    137.12, 138.12,
];

fn main() {
    let drawing_area = BitMapBackend::new("images/3.png", (600, 400)).into_drawing_area();

    drawing_area.fill(&WHITE).unwrap();

    let start_date = Utc.ymd(2019, 10, 1);
    let end_date = Utc.ymd(2019, 10, 18);

    let mut chart = ChartBuilder::on(&drawing_area)
        .caption("Timeseries Test", ("sans-serif", 30))
        // enables Y axis, the size is 40 px
        .set_label_area_size(LabelAreaPosition::Left, 30)
        // enable X axis, the size is 40 px
        .set_label_area_size(LabelAreaPosition::Bottom, 25)
        .build_cartesian_2d(start_date..end_date, 130.0..145.0)
        .unwrap();

    // dbg!((0..100).map(|x| (x, 100 - x)).collect::<Vec<_>>());
    // dbg!((-10..=10).map(|x| (x, x * x)).collect::<Vec<_>>());
    // dbg!((1..=10).map(|x| (x, x * x)).collect::<Vec<_>>());
    chart
        // .draw_series(LineSeries::new((0..10).map(|x| (x, x * x)), &BLACK))
        .draw_series(LineSeries::new(
            (0..).zip(DATA.iter()).map(|(idx, price)| {
                let day = (idx / 5) * 7 + idx % 5 + 1;
                let date = Utc.ymd(2019, 10, day);
                (date, *price)
            }),
            &BLUE,
        ))
        .unwrap();

    chart.configure_mesh().draw().unwrap();
}
