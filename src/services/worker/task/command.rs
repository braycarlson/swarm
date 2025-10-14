pub enum TaskCommand<T, R> {
    Cancel(String),
    Execute(String, T, Box<dyn FnOnce(T) -> Result<R, String> + Send + 'static>),
}
