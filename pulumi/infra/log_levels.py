# To learn more about this syntax, see
# https://docs.rs/env_logger/0.9.0/env_logger/#enabling-logging
# Quick reminder:
# ERROR > WARN > INFO > DEBUG > TRACE
RUST_LOG_LEVELS = ",".join(
    [
        "DEBUG",
        "h2=WARN",
        "hyper=WARN",
        "rusoto_core=WARN",
        "rustls=WARN",
        # By default, sqlx outputs an INFO for every query executed.
        # It's very noisy!
        # https://github.com/launchbadge/sqlx/issues/942
        "sqlx::query=WARN",
        # By default, Tower outputs a DEBUG time you make a gRPC call,
        # so set it to something less noisy.
        "tower::buffer::worker=WARN",
    ]
)
PY_LOG_LEVEL = "DEBUG"
