use super::statistics::IndexStatistics;

pub enum IndexResult {
    Completed(usize),
    Error(String),
    Progress(IndexStatistics),
}
