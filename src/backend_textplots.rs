use jiff::{tz::TimeZone, Timestamp};
use miette::Result;
use prometheus_http_query::response::RangeVector;
use rgb::RGB8;
use textplots::{
    Chart, ColorPlot, LabelBuilder, LabelFormat, Shape, TickDisplay, TickDisplayBuilder,
};

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
        if data.is_empty() {
            return Ok("No data".to_string());
        }

        // Collect all series data and find global bounds
        let mut all_series: Vec<(String, Vec<(f64, f64)>)> = Vec::new();
        let mut global_ymin = f64::INFINITY;
        let mut global_ymax = f64::NEG_INFINITY;

        for v in data.iter() {
            let metric_name = v.metric().get("__name__").cloned().unwrap_or_default();
            let job = v.metric().get("job").cloned().unwrap_or_default();
            let instance = v.metric().get("instance").cloned().unwrap_or_default();

            // Create a label for this series
            let series_label = if !job.is_empty() {
                format!("{}(job={})", metric_name, job)
            } else if !instance.is_empty() {
                format!("{}(instance={})", metric_name, instance)
            } else {
                metric_name
            };

            let points: Vec<(f64, f64)> = v
                .samples()
                .into_iter()
                .map(|s| (s.timestamp(), s.value()))
                .collect();

            if !points.is_empty() {
                let (_, _, ymin, ymax) = get_bounds(&points);
                global_ymin = global_ymin.min(ymin);
                global_ymax = global_ymax.max(ymax);

                all_series.push((series_label, points));
            }
        }

        if all_series.is_empty() {
            return Ok("No valid data points".to_string());
        }

        // Define colors for different series
        let colors = [
            RGB8::new(252, 0, 0),   // Red
            RGB8::new(0, 252, 0),   // Green
            RGB8::new(0, 0, 252),   // Blue
            RGB8::new(252, 252, 0), // Yellow
            RGB8::new(252, 0, 252), // Magenta
            RGB8::new(0, 252, 252), // Cyan
            RGB8::new(252, 165, 0), // Orange
            RGB8::new(128, 0, 128), // Purple
        ];

        // Find the max number of points for consistent X-axis
        let max_points = all_series
            .iter()
            .map(|(_, points)| points.len())
            .max()
            .unwrap_or(0);

        // Create single chart
        let mut chart = Chart::new_with_y_range(
            self.width,
            self.height,
            0.0,
            max_points as f32,
            (global_ymin - 0.01) as f32,
            (global_ymax + 0.01) as f32,
        );

        println!("Plotting {} series on single chart:", all_series.len());

        // Prepare data for plotting - clone everything needed for closures upfront
        let mut shapes_and_colors: Vec<(Shape, RGB8)> = Vec::new();

        for (i, (series_label, points)) in all_series.iter().enumerate() {
            let color = colors[i % colors.len()];
            let vec32: Vec<(f32, f32)> =
                points.iter().map(|(x, y)| (*x as f32, *y as f32)).collect();

            println!(
                "- {}: {} points (color: RGB({}, {}, {}))",
                series_label,
                points.len(),
                color.r,
                color.g,
                color.b
            );

            let shape = Shape::Continuous(Box::new(move |x| {
                let idx = x as usize;
                if idx < vec32.len() {
                    vec32[idx].1
                } else {
                    0.0
                }
            }));

            shapes_and_colors.push((shape, color));
        }

        // Plot all series on the chart
        let mut chart_ptr = &mut chart;
        for (shape, color) in shapes_and_colors.iter() {
            chart_ptr = chart_ptr.linecolorplot(shape, *color);
        }

        // Use the first series for time labels
        if let Some((_, first_points)) = all_series.first() {
            let points_clone = first_points.clone();
            chart_ptr
                .x_label_format(LabelFormat::Custom(Box::new(move |val| {
                    let idx = val as usize;
                    if idx < points_clone.len() {
                        let ts = Timestamp::from_second(points_clone[idx].0 as i64).unwrap();
                        let zoned = ts.to_zoned(TimeZone::system());
                        zoned.strftime("%H:%M").to_string()
                    } else if !points_clone.is_empty() {
                        let ts =
                            Timestamp::from_second(points_clone[points_clone.len() - 1].0 as i64)
                                .unwrap();
                        let zoned = ts.to_zoned(TimeZone::system());
                        zoned.strftime("%H:%M").to_string()
                    } else {
                        "N/A".to_string()
                    }
                })))
                .y_label_format(LabelFormat::Value)
                .y_tick_display(TickDisplay::Sparse)
                .display();
        }

        Ok(format!("Displayed {} series", all_series.len()))
    }
}
