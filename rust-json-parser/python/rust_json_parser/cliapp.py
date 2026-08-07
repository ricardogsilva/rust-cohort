from cyclopts import App
from cyclopts.types import StdioPath

from . import dumps, parse_json

app = App()


@app.default
def parse_input(input_: StdioPath, indent: int | None = None) -> None:
    """Parses either a JSON file or a piped JSON snippet. Piped data must be prefixed by '-'"""
    raw = input_.read_text()
    parsed = parse_json(raw)
    print(dumps(parsed, indent=indent))
