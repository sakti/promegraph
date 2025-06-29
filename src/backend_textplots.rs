use jiff::{tz::TimeZone, Timestamp};
use miette::Result;
use owo_colors::{OwoColorize, Rgb};
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
        let mut global_time_min = f64::INFINITY;
        let mut global_time_max = f64::NEG_INFINITY;

        for v in data.iter() {
            let metric_name = v.metric().get("__name__").cloned().unwrap_or_default();

            // Create a label for this series by finding distinguishing labels
            let mut label_parts = Vec::new();

            // Get all metric labels except __name__
            let mut metric_labels: Vec<(String, String)> = v
                .metric()
                .iter()
                .filter(|(key, _)| *key != "__name__")
                .map(|(key, value)| (key.clone(), value.clone()))
                .collect();

            // Sort for consistent ordering
            metric_labels.sort_by(|a, b| a.0.cmp(&b.0));

            // Build label string from all available labels
            for (key, value) in metric_labels {
                if !value.is_empty() {
                    label_parts.push(format!("{}={}", key, value));
                }
            }

            let series_label = if label_parts.is_empty() {
                metric_name
            } else {
                format!("{}({})", metric_name, label_parts.join(","))
            };

            let points: Vec<(f64, f64)> = v
                .samples()
                .into_iter()
                .map(|s| (s.timestamp(), s.value()))
                .collect();

            if !points.is_empty() {
                let (xmin, xmax, ymin, ymax) = get_bounds(&points);
                global_ymin = global_ymin.min(ymin);
                global_ymax = global_ymax.max(ymax);
                global_time_min = global_time_min.min(xmin);
                global_time_max = global_time_max.max(xmax);

                all_series.push((series_label, points));
            }
        }

        if all_series.is_empty() {
            return Ok("No valid data points".to_string());
        }

        // Define colors for different series
        let colors = [
            RGB8::new(0, 252, 0),   // Green
            RGB8::new(252, 0, 0),   // Red
            RGB8::new(252, 252, 0), // Yellow
            RGB8::new(252, 0, 252), // Magenta
            RGB8::new(0, 252, 252), // Cyan
            RGB8::new(252, 165, 0), // Orange
            RGB8::new(128, 0, 128), // Purple
            RGB8::new(0, 0, 252),   // Blue
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

        println!("Plotting {} series:", all_series.len());

        // Prepare data for plotting - clone everything needed for closures upfront
        let mut shapes_and_colors: Vec<(Shape, RGB8)> = Vec::new();

        for (i, (series_label, points)) in all_series.iter().enumerate() {
            let color = colors[i % colors.len()];
            let points_clone = points.clone();

            let owo_color = Rgb(color.r, color.g, color.b);

            println!(
                "{}",
                format!(
                    "- {}: {} points (color: RGB({}, {}, {}))",
                    series_label,
                    points.len(),
                    color.r,
                    color.g,
                    color.b
                )
                .color(owo_color)
            );

            let shape = Shape::Continuous(Box::new(move |x| {
                let chart_idx = x as usize;

                if points_clone.is_empty() || chart_idx >= max_points {
                    return 0.0;
                }

                // Map chart index to this series' time range
                let time_progress = chart_idx as f64 / (max_points - 1).max(1) as f64;
                let target_time =
                    global_time_min + time_progress * (global_time_max - global_time_min);

                // Find the closest data points for interpolation
                let mut left_idx = None;
                let mut right_idx = None;

                for (idx, &(time, _)) in points_clone.iter().enumerate() {
                    if time <= target_time {
                        left_idx = Some(idx);
                    }
                    if time >= target_time && right_idx.is_none() {
                        right_idx = Some(idx);
                        break;
                    }
                }

                match (left_idx, right_idx) {
                    (Some(left), Some(right)) if left == right => {
                        // Exact match
                        points_clone[left].1 as f32
                    }
                    (Some(left), Some(right)) => {
                        // Linear interpolation
                        let (t1, y1) = points_clone[left];
                        let (t2, y2) = points_clone[right];
                        if t2 > t1 {
                            let ratio = (target_time - t1) / (t2 - t1);
                            (y1 + ratio * (y2 - y1)) as f32
                        } else {
                            y1 as f32
                        }
                    }
                    (Some(left), None) => {
                        // Use the last available point
                        points_clone[left].1 as f32
                    }
                    (None, Some(right)) => {
                        // Use the first available point
                        points_clone[right].1 as f32
                    }
                    (None, None) => 0.0,
                }
            }));

            shapes_and_colors.push((shape, color));
        }

        // Plot all series on the chart
        let mut chart_ptr = &mut chart;
        for (shape, color) in shapes_and_colors.iter() {
            chart_ptr = chart_ptr.linecolorplot(shape, *color);
        }

        // Use the series with the most points for time labels
        if let Some((_, most_detailed_points)) =
            all_series.iter().max_by_key(|(_, points)| points.len())
        {
            let points_clone = most_detailed_points.clone();
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
