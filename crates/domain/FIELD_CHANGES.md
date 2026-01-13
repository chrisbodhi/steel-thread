# Changing ActuatorPlate Fields

This document lists all the places you need to update when adding, removing, or modifying fields in the `ActuatorPlate` struct.

## Checklist for Adding a New Field

When adding a new field like `bracket_width: Millimeters` to `ActuatorPlate`:

### 1. **Domain Crate** (`crates/domain/`)
- [ ] `src/lib.rs` - Add field to `ActuatorPlate` struct with doc comments
- [ ] `src/lib.rs` - Add parameter to `ActuatorPlate::new()` constructor
- [ ] `src/lib.rs` - Add default value in `ActuatorPlate::default()`

### 2. **Validation Crate** (`crates/validation/`)
- [ ] `src/lib.rs` - Add validation call in `validate()` function
- [ ] `src/lib.rs` - Add individual validator function (e.g., `validate_bracket_width()`)
- [ ] `src/lib.rs` - Add error variant to `PlateValidationError` enum
- [ ] `src/lib.rs` - Add error message in `Display` impl
- [ ] `src/lib.rs` - Add unit test for valid values
- [ ] `src/lib.rs` - Add unit test for invalid values
- [ ] `src/lib.rs` - Update `test_validate_full_plate_valid()` with new field
- [ ] `src/lib.rs` - Update `test_validate_full_plate_invalid_bolt_spacing()` with new field
- [ ] `src/lib.rs` - Update `test_error_display_messages()` with new error message
- [ ] `README.md` - Document the new validator function (if applicable)

### 3. **Web/API Crate** (`crates/web/`)
- [ ] `tests/api_tests.rs` - Add new field to all `ActuatorPlate` test instances:
  - `test_create_plate_valid()`
  - `test_create_plate_invalid_bolt_spacing()`
  - `test_create_plate_all_fields_invalid()`
  - Any other test constructing `ActuatorPlate`

### 4. **Parametric Crate** (`crates/parametric/`)
- [ ] `src/lib.rs` - Add field to `generate_params_file()` format string
- [ ] `src/lib.rs` - Add field reference for the KCL constant (e.g., `plate.bracket_width`)
- [ ] `src/lib.rs` - Update test instances of `ActuatorPlate`
- [ ] `src/main.kcl` - Import new constant from `params.kcl`
- [ ] `src/main.kcl` - Pass new parameter to `plate()` function call
- [ ] `src/plate.kcl` - Add parameter to `plate()` function signature
- [ ] `src/plate.kcl` - Use parameter in CAD operations (if applicable)

### 5. **Frontend** (`frontend/`)
- [ ] Update any form components that construct `ActuatorPlate` (when built)

### 6. **Documentation**
- [ ] `CLAUDE.md` - Update API examples with new field
- [ ] `TESTING.md` - Document new tests added
- [ ] `PLAN.md` - Update if field was planned

### 7. **Run Tests**
- [ ] `just test` - Ensure all tests pass
- [ ] `cargo build` - Ensure compilation succeeds
- [ ] Manual API testing with new field

## Example: Adding `bracket_width`

Here's what changes for adding `bracket_width: Millimeters`:

### Domain (`crates/domain/src/lib.rs`)
```rust
pub struct ActuatorPlate {
    // ... existing fields
    pub bracket_width: Millimeters,  // NEW
}

impl ActuatorPlate {
    pub fn new(
        // ... existing params
        bracket_width: Millimeters,  // NEW
    ) -> Self {
        ActuatorPlate {
            // ... existing fields
            bracket_width,  // NEW
        }
    }

    pub fn default() -> Self {
        ActuatorPlate {
            // ... existing fields
            bracket_width: Millimeters(30),  // NEW
        }
    }
}
```

### Validation (`crates/validation/src/lib.rs`)
```rust
pub fn validate(plate: &ActuatorPlate) -> Result<(), PlateValidationError> {
    // ... existing validations
    validate_bracket_width(plate.bracket_width.0)?;  // NEW
    Ok(())
}

pub fn validate_bracket_width(value: u16) -> Result<(), PlateValidationError> {
    if value == 0 {
        return Err(PlateValidationError::BracketWidthInvalid);
    }
    Ok(())
}

pub enum PlateValidationError {
    // ... existing variants
    BracketWidthInvalid,  // NEW
}

impl core::fmt::Display for PlateValidationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            // ... existing matches
            Self::BracketWidthInvalid => write!(f, "bracket width must be greater than 0"),
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_validate_bracket_width_valid() {
        assert!(validate_bracket_width(30).is_ok());
    }

    #[test]
    fn test_validate_bracket_width_invalid() {
        let result = validate_bracket_width(0);
        assert!(result.is_err());
    }
}
```

### Parametric (`crates/parametric/src/lib.rs`)
```rust
fn generate_params_file(plate: &ActuatorPlate) -> std::io::Result<()> {
    let content = format!(
        "@settings(defaultLengthUnit = mm, kclVersion = 1.0)\n\n\
         // ... existing exports
         export const bracketWidth = {:?}",  // NEW
        // ... existing references
        plate.bracket_width  // NEW
    );
    // ...
}
```

### KCL Files
**`src/main.kcl`:**
```kcl
import plateThickness, boltDiameter, boltSpacing, bracketHeight, pinDiameter, bracketWidth from "params.kcl"

plate(
    plate_thickness = plateThickness,
    // ... existing params
    bracket_width = bracketWidth  // NEW
)
```

**`src/plate.kcl`:**
```kcl
export fn plate(plate_thickness, bolt_spacing, bolt_diameter, bracket_height, pin_diameter, bracket_width) {
    // Use bracket_width in CAD operations
}
```

## Tips

1. **Use Compiler Errors as a Guide**: After adding the field to `ActuatorPlate`, run `cargo build`. The compiler will tell you every place that needs updating.

2. **Search for Field Names**: Use `grep -r "bolt_spacing"` to find all references to existing fields as a template.

3. **Run Tests Incrementally**: Run `cargo test -p validation` after validation changes, then `cargo test -p web` after API changes.

4. **Frontend Last**: Update frontend components after backend is complete and tested.

5. **Default Values**: Choose sensible defaults in `ActuatorPlate::default()` that will pass validation.
