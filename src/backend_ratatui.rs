use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use jiff::{Timestamp, tz::TimeZone};
use miette::{IntoDiagnostic, Result};
use prometheus_http_query::response::RangeVector;
use ratatui::{
    DefaultTerminal,
    layout::{Constraint, Layout},
    style::{Color, Style},
    symbols::Marker,
    text::{Line, Span},
    widgets::{Axis, Block, Chart, Dataset, GraphType, Paragraph, Wrap},
};

use crate::promql;

const COLORS: &[(u8, u8, u8)] = &[
    (0, 252, 0),   // Green
    (252, 0, 0),   // Red
    (252, 252, 0), // Yellow
    (252, 0, 252), // Magenta
    (0, 252, 252), // Cyan
    (252, 165, 0), // Orange
    (128, 0, 128), // Purple
    (0, 0, 252),   // Blue
];

pub struct BackendRatatui {
    addr: String,
    expr: String,
    step: f64,
    duration: u16,
    refresh: u64,
}

struct SeriesData {
    label: String,
    points: Vec<(f64, f64)>,
}

impl SeriesData {
    fn stats(&self) -> (f64, f64, f64) {
        let mut min = f64::INFINITY;
        let mut max = f64::NEG_INFINITY;
        for &(_, y) in &self.points {
            min = min.min(y);
            max = max.max(y);
        }
        let last = self.points.last().map(|&(_, y)| y).unwrap_or(0.0);
        (min, max, last)
    }
}

impl BackendRatatui {
    pub fn new(addr: String, expr: String, step: f64, duration: u16, refresh: u64) -> Self {
        Self {
            addr,
            expr,
            step,
            duration,
            refresh,
        }
    }

    pub async fn run(&self) -> Result<()> {
        let mut terminal = ratatui::init();
        let result = self.event_loop(&mut terminal).await;
        ratatui::restore();
        result
    }

    async fn event_loop(&self, terminal: &mut DefaultTerminal) -> Result<()> {
        let refresh_interval = Duration::from_secs(self.refresh);
        let mut last_fetch = Instant::now();
        let mut series = self.fetch_series().await?;

        loop {
            self.draw(terminal, &series)?;

            if event::poll(Duration::from_millis(250)).into_diagnostic()? {
                if let Event::Key(key) = event::read().into_diagnostic()? {
                    if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') {
                        return Ok(());
                    }
                }
            }

            if last_fetch.elapsed() >= refresh_interval {
                if let Ok(new_series) = self.fetch_series().await {
                    series = new_series;
                }
                last_fetch = Instant::now();
            }
        }
    }

    async fn fetch_series(&self) -> Result<Vec<SeriesData>> {
        let data = promql::get_data(&self.addr, &self.expr, self.step, self.duration).await?;
        Ok(parse_series(&data))
    }

    fn draw(&self, terminal: &mut DefaultTerminal, series: &[SeriesData]) -> Result<()> {
        terminal
            .draw(|frame| {
                if series.is_empty() {
                    return;
                }

                // Legend height: one line per series, plus 2 for border
                let legend_height = (series.len() as u16 + 2).min(frame.area().height / 3);
                let chunks =
                    Layout::vertical([Constraint::Min(8), Constraint::Length(legend_height)])
                        .split(frame.area());

                // Compute global bounds
                let mut x_min = f64::INFINITY;
                let mut x_max = f64::NEG_INFINITY;
                let mut y_min = f64::INFINITY;
                let mut y_max = f64::NEG_INFINITY;

                for s in series {
                    for &(x, y) in &s.points {
                        x_min = x_min.min(x);
                        x_max = x_max.max(x);
                        y_min = y_min.min(y);
                        y_max = y_max.max(y);
                    }
                }

                // Add small padding to y bounds
                let y_padding = (y_max - y_min).abs() * 0.05;
                if y_padding == 0.0 {
                    y_min -= 0.5;
                    y_max += 0.5;
                } else {
                    y_min -= y_padding;
                    y_max += y_padding;
                }

                // Build datasets
                let datasets: Vec<Dataset> = series
                    .iter()
                    .enumerate()
                    .map(|(i, s)| {
                        let (r, g, b) = COLORS[i % COLORS.len()];
                        Dataset::default()
                            .name(s.label.clone())
                            .marker(Marker::Braille)
                            .graph_type(GraphType::Line)
                            .style(Style::default().fg(Color::Rgb(r, g, b)))
                            .data(&s.points)
                    })
                    .collect();

                // Format time labels
                let fmt_time = |ts: f64| -> String {
                    if let Ok(t) = Timestamp::from_second(ts as i64) {
                        let zoned = t.to_zoned(TimeZone::system());
                        zoned.strftime("%H:%M").to_string()
                    } else {
                        "N/A".to_string()
                    }
                };

                let x_labels = vec![
                    Span::raw(fmt_time(x_min)),
                    Span::raw(fmt_time((x_min + x_max) / 2.0)),
                    Span::raw(fmt_time(x_max)),
                ];

                let y_labels = vec![
                    Span::raw(format!("{:.2}", y_min)),
                    Span::raw(format!("{:.2}", (y_min + y_max) / 2.0)),
                    Span::raw(format!("{:.2}", y_max)),
                ];

                let title = format!(
                    " {} | refresh: {}s | press q to quit ",
                    self.expr, self.refresh
                );

                let chart = Chart::new(datasets)
                    .block(Block::bordered().title(title))
                    .x_axis(
                        Axis::default()
                            .title("Time")
                            .bounds([x_min, x_max])
                            .labels(x_labels),
                    )
                    .y_axis(
                        Axis::default()
                            .title("Value")
                            .bounds([y_min, y_max])
                            .labels(y_labels),
                    );

                frame.render_widget(chart, chunks[0]);

                // Render legend as a separate widget
                let legend_lines: Vec<Line> = series
                    .iter()
                    .enumerate()
                    .map(|(i, s)| {
                        let (r, g, b) = COLORS[i % COLORS.len()];
                        let color = Color::Rgb(r, g, b);
                        let (min, max, last) = s.stats();
                        Line::from(vec![
                            Span::styled("■ ", Style::default().fg(color)),
                            Span::raw(format!(
                                "{} | min: {:.2}  max: {:.2}  last: {:.2}",
                                s.label, min, max, last
                            )),
                        ])
                    })
                    .collect();

                let legend = Paragraph::new(legend_lines)
                    .block(Block::bordered().title(" Legend "))
                    .wrap(Wrap { trim: false });

                frame.render_widget(legend, chunks[1]);
            })
            .into_diagnostic()?;
        Ok(())
    }
}

fn parse_series(data: &[RangeVector]) -> Vec<SeriesData> {
    data.iter()
        .map(|v| {
            let metric_name = v.metric().get("__name__").cloned().unwrap_or_default();

            let label_parts: Vec<String> = v
                .metric()
                .iter()
                .filter(|(key, _)| *key != "__name__")
                .map(|(key, value)| (key.clone(), value.clone()))
                .collect::<Vec<_>>()
                .tap_sort()
                .iter()
                .filter(|(_, value)| !value.is_empty())
                .map(|(key, value)| format!("{}={}", key, value))
                .collect();

            let label = if label_parts.is_empty() {
                metric_name
            } else {
                format!("{}({})", metric_name, label_parts.join(","))
            };

            let points: Vec<(f64, f64)> = v
                .samples()
                .iter()
                .map(|s| (s.timestamp(), s.value()))
                .collect();

            SeriesData { label, points }
        })
        .filter(|s| !s.points.is_empty())
        .collect()
}

trait TapSort {
    fn tap_sort(&mut self) -> &Self;
}

impl<T: Ord> TapSort for Vec<T> {
    fn tap_sort(&mut self) -> &Self {
        self.sort();
        self
    }
}
