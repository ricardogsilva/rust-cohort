from pathlib import Path

from cyclopts import App
from cyclopts.types import StdioPath

from . import (
    dumps,
    parse_json,
    performance_benchmarks,
)

app = App()


@app.default
def parse_input(input_: StdioPath, indent: int | None = None) -> None:
    """Parses either a JSON file or a piped JSON snippet. Piped data must be prefixed by '-'"""
    raw = input_.read_text()
    parsed = parse_json(raw)
    print(dumps(parsed, indent=indent))


@app.command()
def benchmark(*sample_json: Path, num_iterations: int = 20_000) -> None:
    """Benchmark the performance of Rust JSON parser.

    Provide a set of input sample JSON data and have it be parsed 'num_iterations' times, with time measurement.
    This also executes both Python's stdlib 'json' module and the third-party 'simplejson' package and provides a
    comparison of all three implementations' execution times.
    """
    if len(sample_json) == 0:
        raise RuntimeError("No sample JSON files provided for benchmarking.")

    if not all(entry.is_file() for entry in sample_json):
        raise RuntimeError("All provided sample JSON paths must be files.")

    result = performance_benchmarks.BenchmarkCollection()
    for entry in sample_json:
        app.error_console.print(f"Processing {entry}...")
        result.items.append(
            performance_benchmarks.perform_benchmark(
                json_str=entry.read_text(),
                name=entry.stem,
                num_iterations=num_iterations,
            )
        )
    app.console.print(result)
