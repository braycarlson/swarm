pub enum TaskResult<R> {
    Completed(String, R),
    Error(String, String),
    Started(String),
}
