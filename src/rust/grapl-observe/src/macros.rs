#[macro_export]
macro_rules! log_time {
    ($msg:expr, $x:expr) => {{
        let mut sw = stopwatch::Stopwatch::start_new();
        #[allow(path_statements)]
        let result = $x;
        sw.stop();
        info!("{} {} milliseconds", $msg, sw.elapsed_ms());
        result
    }};
}
