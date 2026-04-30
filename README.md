# Platano

## Structure 
```
platano/
├── Cargo.toml
├── src/
│   ├── main.rs           # entry point, minimal
│   ├── lib.rs            # re-exports
│   ├── config.rs         # env + config loading
│   ├── llm/              # provider client (start with one)
│   │   └── mod.rs
│   ├── agent/            # core loop
│   │   └── mod.rs
│   └── telemetry.rs      # tracing setup
├── tests/
│   └── health.rs         # one passing integration test
└── README.md
```
