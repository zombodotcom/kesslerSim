# Development Guide

## Quick Testing Workflow

### Fastest Feedback Loop

**1. Quick Compile Check (Fastest)**
```bash
cargo check
```
- Skips codegen, only checks compilation
- Usually completes in 1-3 seconds
- Use this for rapid iteration

**2. Run Specific Tests**
```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run tests with output
cargo test -- --nocapture

# Run only library tests (faster)
cargo test --lib
```

**3. Watch Mode (Auto-check on save)**
```bash
# Install cargo-watch if not already installed
cargo install cargo-watch

# Watch and auto-check
cargo watch -x check

# Watch and auto-test
cargo watch -x test
```

### Windows PowerShell Scripts

Use the provided `scripts/dev.ps1` script:

```powershell
# Quick compile check
.\scripts\dev.ps1 check

# Watch for changes
.\scripts\dev.ps1 watch

# Run tests
.\scripts\dev.ps1 test

# Run the app
.\scripts\dev.ps1 run
```

### Linux/Mac Make Commands

```bash
# Quick compile check
make check

# Watch for changes
make watch

# Run tests
make test

# Run the app
make run
```

## Performance Tips

### 1. Use Incremental Compilation
Already configured in `.cargo/config.toml`:
- Incremental compilation enabled
- Parallel compilation (4 jobs)
- Optimized debug builds (opt-level = 1)

### 2. Profile-Specific Optimization
In `Cargo.toml`:
- `dev` profile: opt-level = 1 (faster than 0, still debuggable)
- `dev.package.*`: opt-level = 3 (optimize dependencies)
- `release`: opt-level = 3, LTO disabled (faster release builds)

### 3. Test Only What You Need
```bash
# Test specific module
cargo test --lib gpu_instancing

# Test specific file
cargo test --test test_name

# Skip expensive tests
cargo test -- --skip expensive_test
```

## Debugging Tips

### 1. Use `cargo check` Instead of `cargo build`
- Much faster (skips codegen)
- Still catches all compilation errors
- Perfect for TDD workflow

### 2. Conditional Compilation
```rust
#[cfg(test)]
mod tests {
    // Test-only code
}
```

### 3. Debug Prints
```rust
// Use dbg!() for quick debugging
dbg!(variable);

// Or conditional debug prints
#[cfg(debug_assertions)]
println!("Debug: {:?}", value);
```

## Common Workflows

### TDD Workflow
1. Write test: `cargo test test_name -- --nocapture`
2. Check compiles: `cargo check`
3. Implement: Write code
4. Verify: `cargo test test_name`
5. Refactor: `cargo check` after each change

### Feature Development
1. Create feature branch
2. Write tests first
3. Implement feature
4. Run full test suite: `cargo test`
5. Check for warnings: `cargo clippy`
6. Format code: `cargo fmt`

### Performance Testing
```bash
# Run benchmarks
cargo bench

# Profile with perf (Linux)
perf record cargo run --release
perf report

# Profile with Instruments (Mac)
# Use Xcode Instruments
```

## Troubleshooting

### Slow Compilation
1. Check `target/` directory size - clean if too large: `cargo clean`
2. Ensure incremental compilation is enabled (in `.cargo/config.toml`)
3. Use `cargo check` instead of `cargo build` during development

### Test Failures
1. Run with `--nocapture` to see output: `cargo test -- --nocapture`
2. Run specific test: `cargo test test_name`
3. Check test fixtures are present

### Type Errors
1. Use `cargo check` for fast feedback
2. Check Bevy version compatibility
3. Use `cargo doc --open` to browse API docs

## IDE Integration

### VS Code
- Install "rust-analyzer" extension
- Enable "check on save"
- Use "Run Test" code lens above tests

### CLion / IntelliJ
- Install Rust plugin
- Enable "Build on save"
- Use "Run" gutter icons for tests

## Continuous Integration

The project is set up for CI/CD with:
- Fast incremental builds
- Parallel test execution
- Cached dependencies

For local CI simulation:
```bash
# Run full CI checks
cargo check && cargo test && cargo clippy && cargo fmt --check
```

