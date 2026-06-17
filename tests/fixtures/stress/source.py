from dataclasses import dataclass


@dataclass
class Options:
    retries: int


def traced(fn):
    return fn


class Service:
    class Inner:
        def normalize(self, value: str) -> str:
            return value.strip()

    def __init__(self, options: Options):
        self.options = options

    @traced
    async def start(self, values: list[str]) -> list[str]:
        normalize = lambda value: self.Inner().normalize(value)

        def filter_value(value: str) -> bool:
            return bool(value)

        return [normalize(value) for value in values if filter_value(value)]


def build_service(retries: int = 3) -> Service:
    return Service(Options(retries=retries))
