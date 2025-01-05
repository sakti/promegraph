use plotters::prelude::*;

fn main() {
    let drawing_area = BitMapBackend::new("images/2.png", (600, 400)).into_drawing_area();

    drawing_area.fill(&WHITE).unwrap();

    let mut chart = ChartBuilder::on(&drawing_area)
        .caption("Prometheus Graph", ("Arial", 30))
        // enables Y axis, the size is 40 px
        .set_label_area_size(LabelAreaPosition::Left, 30)
        // enable X axis, the size is 40 px
        .set_label_area_size(LabelAreaPosition::Bottom, 25)
        .build_cartesian_2d(0..10, 0..10)
        .unwrap();

    // dbg!((0..100).map(|x| (x, 100 - x)).collect::<Vec<_>>());
    // dbg!((-10..=10).map(|x| (x, x * x)).collect::<Vec<_>>());
    dbg!((1..=10).map(|x| (x, x * x)).collect::<Vec<_>>());
    chart
        // .draw_series(LineSeries::new((0..10).map(|x| (x, x * x)), &BLACK))
        .draw_series(
            AreaSeries::new((0..=10).map(|x| (x, x * x)), 0, &BLUE.mix(0.3)).border_style(&BLUE),
        )
        .unwrap();

    chart.configure_mesh().draw().unwrap();
}
