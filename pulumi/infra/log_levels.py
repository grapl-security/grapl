# To learn more about this syntax, see
# https://docs.rs/env_logger/0.9.0/env_logger/#enabling-logging
RUST_LOG_LEVELS = ",".join(
    [
        "DEBUG",
        "h2=WARN",
        "hyper=WARN",
        "rusoto_core=WARN",
        "rustls=WARN",
        # noisy, only for debugging
        "client_executor=TRACE",
        # By default, sqlx outputs an INFO for every query executed.
        # It's very noisy!
        # https://github.com/launchbadge/sqlx/issues/942
        "sqlx::query=WARN",

        # By default, Tower outputs every time you make a gRPC call.
        "tower::buffer::worker=WARN",
    ]
)
PY_LOG_LEVEL = "DEBUG"
