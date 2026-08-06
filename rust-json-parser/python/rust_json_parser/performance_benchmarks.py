import dataclasses

from rich.table import Table

from . import benchmark_performance


@dataclasses.dataclass(frozen=True)
class BenchmarkResult:
    name: str
    json_size_kb: float
    num_iterations: int
    rust_secs: float
    stdlib_secs: float
    simplejson_secs: float
    rust_vs_stdlib: float
    rust_vs_simplejson: float


@dataclasses.dataclass(frozen=True)
class BenchmarkCollection:
    items: list[BenchmarkResult] = dataclasses.field(default_factory=list)

    def __rich__(self):
        result = Table(title="Benchmark result")
        result.add_column("Item")
        result.add_column("Size (KB)")
        result.add_column("Iterations")
        result.add_column("native Python json (s)")
        result.add_column("simplejson (s)")
        result.add_column("rust-json-parser (s)")
        result.add_column("vs stdlib (%)")
        result.add_column("vs simplejson (%)")
        for i in self.items:
            result.add_row(
                i.name,
                format(i.json_size_kb, ".2f"),
                format(i.num_iterations, "_d"),
                str(i.stdlib_secs),
                str(i.simplejson_secs),
                str(i.rust_secs),
                format(i.rust_vs_stdlib, ".4f"),
                format(i.rust_vs_simplejson, ".4f"),
                # str(i.rust_vs_stdlib),
                # str(i.rust_vs_simplejson),
            )
        return result


def perform_benchmark(
    json_str: str, name: str = "benchmark", num_iterations: int = 10_000
) -> BenchmarkResult:
    rust_secs, stdlib_secs, simplejson_secs = benchmark_performance(
        json_str=json_str, iterations=num_iterations
    )
    return BenchmarkResult(
        name=name,
        json_size_kb=len(json_str.encode("utf-8")) / 1024,
        num_iterations=num_iterations,
        rust_secs=rust_secs,
        stdlib_secs=stdlib_secs,
        simplejson_secs=simplejson_secs,
        rust_vs_stdlib=(stdlib_secs - rust_secs) / rust_secs * 100,
        rust_vs_simplejson=(simplejson_secs - rust_secs) / rust_secs * 100,
    )
