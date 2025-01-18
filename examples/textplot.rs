use textplots::{Chart, Plot, Shape};

fn main() {
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
    Chart::new(300, 100, -10.0, 10.0)
        .lineplot(&Shape::Lines(&points))
        .display();

    println!("\ny = staircase points");
    Chart::default().lineplot(&Shape::Steps(&points)).display();

    println!("\ny = scatter plot");
    Chart::default().lineplot(&Shape::Points(&points)).display();
}
