from typing_extensions import TypedDict


class NomadServicePostgresDbArgs(TypedDict):
    hostname: str
    port: int
    username: str
    password: str
