# mycelium

`mycelium` is a `no_std`-friendly Rust framework for building modular,
distributed applications over DDS. It provides provider and consumer modules,
runtime context integration, and the `#[provides]` and `#[consumes]` attribute
macros.

## Installation

```toml
[dependencies]
mycelium = { version = "0.0.1", features = ["std_runtime"] }
dust_dds = "0.15.0"
```

The standard runtime is optional. Disable `std_runtime` when supplying a
different `RuntimeContext` implementation.

See the [repository README](https://github.com/Mycellium-Computing/mycelium_rs)
for examples and the framework documentation.
