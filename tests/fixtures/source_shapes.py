class Service:
    label: str

    def __init__(self, label: str) -> None:
        self.label = label

    async def start(self, retries: int = 1) -> str:
        return self.label

def build_service(label: str) -> Service:
    return Service(label)
