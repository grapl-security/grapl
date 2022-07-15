# To learn more about this syntax, see
# https://docs.rs/env_logger/0.9.0/env_logger/#enabling-logging
RUST_LOG_LEVELS = ",".join(
    [
        "DEBUG",
        "h2=WARN",
        "hyper=WARN",
        "rusoto_core=WARN",
        "rustls=WARN",
        "tower=WARN",
    ]
)
PY_LOG_LEVEL = "DEBUG"
