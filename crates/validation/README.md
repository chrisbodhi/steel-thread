# Validation Crate

This crate provides validation logic for actuator plate parameters. It's designed to work in both native Rust (backend) and WebAssembly (frontend) environments.

## Architecture

The validation crate is `no_std` compatible, making it suitable for WebAssembly compilation. It provides:

1. **Core validation functions** - Individual validators for each plate parameter
2. **Full plate validation** - Validates all parameters at once
3. **WASM bindings** - JavaScript-friendly validation functions

## Usage

### In Rust (Backend)

```rust
use validation::{validate_bolt_spacing, PlateValidationError};

match validate_bolt_spacing(60) {
    Ok(()) => println!("Valid!"),
    Err(e) => println!("Error: {}", e),
}
```

### In JavaScript/TypeScript (Frontend)

The validation crate is compiled to WebAssembly and exposed through a TypeScript wrapper:

```typescript
import { validateBoltSpacing } from './lib/validation';

const result = await validateBoltSpacing(60);
if (result.valid) {
  console.log('Valid!');
} else {
  console.error('Error:', result.error);
}
```

## Building for WebAssembly

Build the WASM module:

```bash
wasm-pack build crates/validation --target web --out-dir ../../frontend/src/wasm-validation --no-typescript
```

Or use the just command:

```bash
just build-wasm
```

The WASM module is automatically built as part of:
- `just build` (frontend build)
- `just dev` (development mode)
- `just build-release` (production build)

## Validation Rules

### Bolt Spacing
- Must be greater than 0

### Bolt Diameter
- Must be greater than 0

### Bracket Height
- Must be greater than 0

### Bracket Width
- Must be greater than 0

### Pin Diameter
- Must be greater than 0

### Pin Count
- Must be at least 1
- Must not exceed 12

### Plate Thickness
- Must be greater than 0

## Development

Run tests:

```bash
cargo test -p validation
```

The WASM module provides real-time validation in the frontend for a tight feedback loop, while the backend also validates to ensure security.
