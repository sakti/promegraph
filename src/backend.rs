use miette::Result;
use prometheus_http_query::response::RangeVector;

pub trait Generator {
    fn generate(&self, data: Vec<RangeVector>) -> Result<String>;
}
