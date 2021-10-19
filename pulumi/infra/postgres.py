import os


def get_local_postgres_url() -> str:
    user = os.environ["POSTGRES_USER"]
    password = os.environ["POSTGRES_PASSWORD"]
    db_name = os.environ["POSTGRES_DB"]

    return f"postgres://{user}:{password}@LOCAL_GRAPL_REPLACE_IP:5432/{db_name}"
