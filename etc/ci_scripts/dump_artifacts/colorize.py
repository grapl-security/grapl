class Colorize:
    GREEN = "\x1b[32m"
    RESET = "\x1b[0m"

    @classmethod
    def _colorize(cls, escape: str, message: str) -> str:
        return f"{escape}{message}{cls.RESET}"

    @classmethod
    def green(cls, message: str) -> str:
        return cls._colorize(cls.GREEN, message)
