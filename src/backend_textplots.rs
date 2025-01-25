use jiff::{tz::TimeZone, Timestamp};
use miette::Result;
use prometheus_http_query::response::RangeVector;
use textplots::{Chart, LabelBuilder, LabelFormat, Plot, Shape, TickDisplay, TickDisplayBuilder};

use crate::backend::Generator;

pub struct BackendTextplots {
    width: u32,
    height: u32,
}

impl BackendTextplots {
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }
}

fn get_bounds(points: &[(f64, f64)]) -> (f64, f64, f64, f64) {
    if points.is_empty() {
        return (0.0, 0.0, 0.0, 0.0);
    }

    let mut xmin = points[0].0;
    let mut xmax = points[0].0;
    let mut ymin = points[0].1;
    let mut ymax = points[0].1;

    for &(x, y) in points.iter() {
        xmin = xmin.min(x);
        xmax = xmax.max(x);
        ymin = ymin.min(y);
        ymax = ymax.max(y);
    }

    (xmin, xmax, ymin, ymax)
}

impl Generator for BackendTextplots {
    fn generate(&self, data: Vec<RangeVector>) -> Result<String> {
        for (_i, v) in data.iter().enumerate() {
            let _metric_name = v.metric().get("__name__").cloned().unwrap_or_default();
            let points: Vec<(f64, f64)> = v
                .samples()
                .into_iter()
                .map(|s| (s.timestamp(), s.value()))
                .collect();
            let (_xmin, _xmax, ymin, ymax) = get_bounds(&points);

            let vec32: Vec<(f32, f32)> =
                points.iter().map(|(x, y)| (*x as f32, *y as f32)).collect();

            Chart::new_with_y_range(
                self.width,
                self.height,
                0.0,
                points.len() as f32,
                (ymin - 0.01) as f32,
                (ymax + 0.01) as f32,
            )
            .lineplot(&Shape::Continuous(Box::new(|x| {
                let idx = x as usize;
                if idx < vec32.len() {
                    vec32[idx].1
                } else {
                    0.0
                }
            })))
            .x_label_format(LabelFormat::Custom(Box::new(move |val| {
                let idx = val as usize;
                if idx < points.len() {
                    let ts = Timestamp::from_second(points[idx].0 as i64).unwrap();
                    ts.to_zoned(TimeZone::system()).to_string()
                    // format!("{}", points[idx].0)
                } else {
                    // get last value
                    // format!("{}", points[points.len() - 1].0)
                    let ts = Timestamp::from_second(points[points.len() - 1].0 as i64).unwrap();
                    ts.to_zoned(TimeZone::system()).to_string()
                    // format!("{}", val as usize)
                }
                // format!("{}", points[val as usize].0)
                // format!("{}", val as usize)
            })))
            .y_label_format(LabelFormat::Value)
            .y_tick_display(TickDisplay::Sparse)
            .display();
        }
        Ok("ok".to_string())
    }
}
