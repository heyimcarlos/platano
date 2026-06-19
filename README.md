# Platano

Platano is a Rust local-agent harness around an Ollama chat model. It provides a small agent loop, tool definitions, sandboxed file tools, config loading, and tracing.

## Structure

```text
platano/
├── Cargo.toml
├── config/
│   ├── base.yaml
│   ├── local.yaml
│   └── production.yaml
├── src/
│   ├── main.rs              # CLI entry point
│   ├── lib.rs               # crate modules
│   ├── config.rs            # env + config loading
│   ├── startup.rs           # agent construction
│   ├── telemetry.rs         # tracing setup
│   ├── llm/                 # LLM client and request/response types
│   └── agent/               # agent loop and tools
└── README.md
```

## Checks

```bash
cargo fmt --check
cargo test
cargo clippy --workspace --all-targets -- -D warnings
```
