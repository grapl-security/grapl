#[macro_export]
macro_rules! log_time {
    ($msg:expr, $x:expr) => {{
        let start = Instant::now();
        #[allow(path_statements)]
        let result = $x;
        let duration = start.elapsed().as_millis();
        info!("{} {} milliseconds", $msg, duration);
        result
    }};
}
