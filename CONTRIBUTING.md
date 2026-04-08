# Contributing to hashtime

Thank you for your interest in contributing!

## Prerequisites

- Rust 1.80+
- Python 3.8+
- maturin (`pip install maturin`)

## Development Setup

```bash
git clone https://github.com/songxiaocheng/hashtime.git
cd hashtime
cargo build --release

# For Python development
cd python
maturin develop
```

## Project Structure

- `core/` - Core Rust library
- `cli/` - Command-line interface
- `python/` - Python bindings

## Coding Standards

### Rust

- Run `cargo fmt` before committing
- Run `cargo clippy` to check for lint issues
- All tests must pass: `cargo test`
- Add doc comments for public APIs

### Python

- Follow PEP 8 style guidelines
- Use type hints where appropriate

## Running Tests

```bash
# Rust
cargo test

# Python
cd python && pytest
```

## License

By contributing to hashtime, you agree that your contributions will be licensed under the MIT License.
