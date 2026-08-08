# Mycelium Computing

A Rust framework for building modular, distributed applications using DDS (Data Distribution Service) middleware. This framework simplifies the creation of provider-consumer communication patterns through intuitive procedural macros.

## Overview

Mycelium Computing enables developers to create distributed systems with minimal boilerplate by leveraging Rust's procedural macro system. It abstracts the complexity of DDS-based communication, allowing you to focus on your application logic.

### Key Features

- **Declarative Macros**: Use `#[provides]` and `#[consumes]` attributes to define providers and consumers
- **Multiple Communication Patterns**:
  - **RequestResponse**: Traditional request-response with configurable timeout
  - **Response**: One-way response pattern (no input required)
  - **Continuous**: Streaming/pub-sub pattern for real-time data
- **Type-Safe**: Leverages Rust's type system with compile-time verification
- **DDS-Based**: Built on top of Dust DDS for reliable, high-performance communication
- **Async-First**: Fully asynchronous API design

## Project Structure

```
modular_architecture/
├── mycelium/                    # Main library (mycelium)
│   └── src/
│       ├── core/           # Module, listeners, and messages
│       └── utils/          # Utilities (ID generator, storage)
├── macros/                  # Procedural macros library
│   └── src/
│       ├── provider/       # #[provides] macro implementation
│       └── consumer/       # #[consumes] macro implementation
├── tests/                   # Integration tests
├── benchmarking/           # Performance benchmarks
├── usage_examples/         # Example implementations
└── docs/                   # Documentation
```

## Installation

Add the following to your `Cargo.toml`:

```toml
[dependencies]
mycelium = { version = "0.0.1", features = ["std_runtime"] }
dust_dds = { version = "0.15.0", default-features = false, features = ["dcps", "rtps"] }
```

The `mycelium` package re-exports the procedural macros from the companion
`mycelium-computing-macros` package, so applications only need to depend on `mycelium`.

### Publishing

The workspace contains the publishable `mycelium` library and its required
`mycelium-computing-macros` proc-macro package. Publish the macro dependency first, then
the library:

```bash
cargo publish --package mycelium-computing-macros
cargo publish --package mycelium
```

Use `--dry-run` with either command to validate a package without uploading it.

Crates.io names are globally allocated. The `mycelium` name must be available
or belong to your account before the final publish can succeed.

## Quick Start

### Defining Message Types

First, define your message types using the `DdsType` derive macro:

```rust
use dust_dds::infrastructure::type_support::DdsType;

#[derive(DdsType, Debug)]
pub struct ArithmeticRequest {
    pub a: f32,
    pub b: f32,
}

#[derive(DdsType, Debug)]
pub struct Number {
    pub value: f32,
}
```

### Creating a Provider

Use the `#[provides]` macro to define a service provider:

```rust
use mycelium::provides;

#[provides([
    RequestResponse("add_two_ints", ArithmeticRequest, Number),
    Continuous("stream_data", SensorData)
])]
struct CalculatorProvider;

// Implement the generated trait
impl CalculatorProviderProviderTrait for CalculatorProvider {
    async fn add_two_ints(request: ArithmeticRequest) -> Number {
        Number {
            value: request.a + request.b,
        }
    }
}
```

### Creating a Consumer

Use the `#[consumes]` macro to define a service consumer:

```rust
use mycelium::consumes;

#[consumes([
    RequestResponse("add_two_ints", ArithmeticRequest, Number),
    Continuous("stream_data", SensorData)
])]
struct CalculatorConsumer;

// For continuous data, implement the callback trait
impl CalculatorConsumerContinuosTrait for CalculatorConsumer {
    async fn stream_data(data: SensorData) {
        println!("Received sensor data: {:?}", data);
    }
}
```

The runtime is selected when a module is created, not in the provider or consumer declaration.
For the standard runtime, construct the module with `StdRuntimeContext::new()`.

### Running Provider and Consumer

**Provider Module:**

```rust
use mycelium::core::module::Module;
use mycelium::runtimes::StdRuntimeContext;

async fn run_provider() {
    let mut app = Module::new(
        0,
        "CalculatorService",
        StdRuntimeContext::new(),
    )
    .await;
    
    // Register provider and get handle for continuous data
    let continuous_handle = app.register_provider::<CalculatorProvider>().await;
    
    // Publish continuous data when needed
    continuous_handle.stream_data(&SensorData { /* ... */ }).await;
    
    // Keep provider running
    app.run_forever().await;
}
```

**Consumer Module:**

```rust
use mycelium::core::module::Module;
use mycelium::runtimes::StdRuntimeContext;

async fn run_consumer() {
    let mut app = Module::new(
        0,
        "CalculatorService",
        StdRuntimeContext::new(),
    )
    .await;

    let consumer = app.register_consumer::<CalculatorConsumer>().await;
    
    // Make a request with timeout
    let result = consumer
        .add_two_ints(
            ArithmeticRequest { a: 1.0, b: 2.0 },
            Duration::new(10, 0),
        )
        .await;
    
    match result {
        Some(response) => println!("Result: {}", response.value),
        None => println!("Request timed out"),
    }
}
```

## Communication Patterns

### RequestResponse

A bidirectional pattern where the consumer sends a request and waits for a response:

```rust
#[provides([
    RequestResponse("service_name", RequestType, ResponseType)
])]
```

### Response

A pattern where the provider returns data without requiring input:

```rust
#[provides([
    Response("get_status", StatusResponse)
])]
```

### Continuous

A pub-sub pattern for streaming data from provider to consumers:

```rust
#[provides([
    Continuous("telemetry", TelemetryData)
])]
```

## Architecture

The framework follows a provider-consumer architecture built on DDS:

```
┌──────────────┐     ┌─────────────────┐     ┌──────────────┐
│   Provider   │────>│  DDS Middleware │<────│   Consumer   │
│              │<────│  (Topics/QoS)   │────>│              │
└──────────────┘     └─────────────────┘     └──────────────┘
```

Key components:
- **Module**: Manages DDS participants, publishers, and subscribers
- **ProviderTrait**: Generated trait that providers must implement
- **Proxy**: Generated struct for consumers to interact with providers
- **Listeners**: Handle incoming messages and route to implementations

## Running Tests

```bash
cargo test --workspace
```

## Running Benchmarks

```bash
# Request-Response benchmark
cargo run --bin request_response_provider
cargo run --bin request_response_consumer

# Continuous data benchmark
cargo run --bin continuous_provider
cargo run --bin continuous_consumer
```

## Dependencies

| Dependency | Version | Purpose |
|------------|---------|---------|
| dust_dds | 0.15.0 | DDS middleware and runtime integration |
| proc-macro2 | 1.0.103 | Procedural macro support |
| quote | 1.0.41 | Code generation |
| syn | 2.0.108 | Rust syntax parsing |

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

```
Copyright 2026 Juan David Guevara Arévalo

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.
```

## Author

**Juan David Guevara Arévalo**

---

*Built with ❤️ using Rust and DDS*
