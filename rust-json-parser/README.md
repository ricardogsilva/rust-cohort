## Rust crate

This repo contains a rust crate that has a JSON parser implemented in Rust. The crate generates Python bindings for
the JSON parser, allowing it to be used in Python code.

Use `cargo` to build, test, show docs, etc. as usual.


## Python package

The repo contains a Python package that uses the rust crate. Checkout the `pyproject.toml` file for more info
on how it is set up. In short, it makes use of [maturin] as the build system for the rust crate, and uses [uv]
as a build tool for the Python package.

[maturin]: https://www.maturin.rs/
[uv]: https://docs.astral.sh/uv/


### development

`uv sync` builds the project with a development build of the rust-json-parser crate. This works because we have
`editable-profile = "dev"` in the `[tool.maturin]` section in `pyproject.toml`. The default sync way of uv is to
perform an editable build of the project.

This means we can simply invoke `uv` as usual.


### Benchmarking

In order to get reliable results we need to ensure a `release` build of the rust crate. This can be achieved in
multiple ways:

The more convenient way is to run the benchmark command directly via uv while passing it the `--no-editable` flag.
This flag causes uv to build the project in release mod and also installing it into the already existing venv.

```shell
uv run --no-editable --reinstall-package \
    rust-json-parser benchmark \
    ../test_data/benchmark-small.json \
    ../test_data/benchmark-medium.json \
    ../test_data/benchmark-large.json
```

Note that the `--reinstall-package` flag is just needed when you want to force uv to reinstall the package 
(and recompile the rust crate), it is not necessary if you just want to run the benchmarks multiple times without
changing the code.

Another option is to use `uv build` - this builds the project in a suitable way and puts a wheel in the `dist`
directory. The wheel can then be installed in a new venv in order to run the benchmarks. Note that this new venv
needs to be created as an additional step, and it also needs to have whatever dependencies the benchmarks
require installed;
