# AGENTS.md

## Code Style

### Formatting (cargo fmt)

Always run `cargo fmt` after modifying Rust code to ensure style compliance.

### No Duplicate Code

Extract shared logic into reusable functions. Do not duplicate code across modules.

### No Warnings or Errors

Code must compile without warnings or errors. Address lint issues before committing.

### Performance

This project is performance-oriented: minimize allocations, avoid unnecessary clones, and prefer move semantics over copies.

### Parallelism

Use `rayon` for parallel iteration.

### Text Files

All plaintext files must end with a newline character. This is required by POSIX, which defines a text file as containing zero or more lines terminated by a newline.

### Naming Conventions

- **Functions & Variables**: snake_case
- **Types & Enums**: PascalCase
- **Constants**: SCREAMING_SCREAM_CASE
- **Modules**: snake_case
