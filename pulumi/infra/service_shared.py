"""
Things like consts around cost tuning that should be shared by both
Lambda and Fargate services.
"""


def get_service_log_retention_days() -> int:
    return 31
