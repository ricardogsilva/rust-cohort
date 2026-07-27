# this file is just a stub with the signatures of the generated python bindings
# it is used by type checkers in order to provide better python hints

type JsonValue = (
    dict[str, JsonValue] | list[JsonValue] | str | int | float | bool | None
)

def parse_json(input: str) -> JsonValue: ...
