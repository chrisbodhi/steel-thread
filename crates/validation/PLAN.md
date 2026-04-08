# Validation Crate Enhancement Plan

## Status

- [x] **Phase 1**: Domain Type Changes — `Newtons` type, material properties, `expected_force_per_pin` field
- [x] **Phase 2**: Validation Logic — stress check functions
- [x] **Phase 3**: WASM Bindings
- [ ] **Phase 4**: Web API Updates
- [ ] **Phase 5**: Frontend Updates
- [ ] **Phase 6**: Testing Strategy

### Phase 1 Notes
- Implemented with Option A (4-bolt assumption, no new bolt_count field)
- `expected_force_per_pin` added as a required field (not `Option`), breaking change accepted
- All 49 existing tests updated and passing
- 8 new domain tests added (material properties, density/yield ordering, force cache key)

### Phase 2 Notes
- All 5 stress checks implemented with integer-only arithmetic (no floating point)
- Uses u64 intermediates to prevent overflow
- `minimum_thickness_mm()` advisory function added for frontend guidance
- 22 new validation tests covering: pass/fail for each check, boundary conditions,
  safety factor verification, material-change flip tests, thickness-fix tests
- Updated web test plates to use `ActuatorPlate::default()` (structurally sound)
- Added `Newtons` to OpenAPI schema registration
- Total test count: 71 (up from 49)

### Phase 3 Notes
- Added `wasm_validate_expected_force()` for single-field force validation
- Added `wasm_validate_stress()` — full stress analysis taking all 9 flat params
  (wasm-bindgen can't pass structs, so it constructs `ActuatorPlate` internally)
- Added `wasm_minimum_thickness()` — advisory minimum thickness computation
- Helper functions `parse_bolt_size()` and `parse_material()` convert strings to enums
- TypeScript wrappers: `validateExpectedForce()`, `validateStress()`, `getMinimumThickness()`
- `validateStress` uses a params object for ergonomic TS usage
- `getMinimumThickness` returns 0 on WASM error (graceful degradation)

---

## Objective

Enhance the validation crate to perform **engineering stress analysis** that accounts for
material properties, plate thickness, and expected operating forces. All force-based checks
use a **2× safety factor** (design force = 2 × expected force).

---

## Current State

The validation crate currently performs basic constraint checks:
- Non-zero checks on dimensions (bolt spacing, bracket height/width, pin diameter, thickness)
- Range check on pin count (1–12)
- Enum validation for bolt size and material (string-based, for WASM)
- `no_std` compatible, compiles to both native Rust and WebAssembly

**What's missing:** No relationship between material choice, plate thickness, and mechanical
loads. A 2mm brass plate and a 20mm carbon steel plate pass the same validation today.

---

## Phase 1: Domain Type Changes (`crates/domain`)

### 1.1 Add `Newtons` Type

A new type-safe wrapper for force values, analogous to `Millimeters`:

```rust
/// Type-safe wrapper for force in Newtons.
#[derive(Copy, Clone, Debug, Deserialize, Serialize, PartialEq, PartialOrd)]
pub struct Newtons(pub u32);
```

Using `u32` allows forces up to ~4.3 MN, well beyond actuator plate use cases.
`u32` rather than `u16` because 65 kN max would be too limiting for some industrial actuators.

### 1.2 Add Material Mechanical Properties

Add `const fn` methods to the `Material` enum returning engineering properties.
Reference alloys for each variant:

| Material | Alloy | Yield (MPa) | Tensile (MPa) | Shear (MPa) | Elastic Modulus (GPa) | Density (kg/m³) |
|---|---|---|---|---|---|---|
| Aluminum | 6061-T6 | 276 | 310 | 207 | 68.9 | 2700 |
| StainlessSteel | 304 | 215 | 505 | 310 | 193 | 8000 |
| CarbonSteel | A36 | 250 | 400 | 230 | 200 | 7850 |
| Brass | C36000 | 124 | 338 | 200 | 97 | 8500 |

New methods on `Material`:

```rust
impl Material {
    /// Yield strength in MPa (0.2% offset).
    pub const fn yield_strength_mpa(&self) -> u16 { ... }

    /// Ultimate tensile strength in MPa.
    pub const fn tensile_strength_mpa(&self) -> u16 { ... }

    /// Shear strength in MPa (approx 0.6 × tensile for most metals).
    pub const fn shear_strength_mpa(&self) -> u16 { ... }

    /// Elastic modulus in MPa (stored as u32 since values exceed u16 range).
    /// E.g., 68_900 for aluminum (68.9 GPa).
    pub const fn elastic_modulus_mpa(&self) -> u32 { ... }

    /// Density in kg/m³.
    pub const fn density_kg_m3(&self) -> u16 { ... }
}
```

All properties use integer types for `no_std` simplicity. The elastic modulus is in MPa
(not GPa) to keep units consistent across all stress calculations without floating-point
conversion. Values returned as `u32` since e.g. 200,000 MPa exceeds `u16::MAX`.

### 1.3 Add `expected_force_per_pin` to `ActuatorPlate`

```rust
pub struct ActuatorPlate {
    // ... existing fields ...

    /// Expected operating force per actuator pin (in Newtons).
    ///
    /// This is the nominal force each pin is expected to handle during
    /// normal operation. Stress checks apply a 2× safety factor internally.
    pub expected_force_per_pin: Newtons,
}
```

Update `Default`:
```rust
expected_force_per_pin: Newtons(500), // 500 N per pin, modest default
```

Update `cache_key()` to include the new field in the SHA-256 hash.

### 1.4 Add Bolt Count Derivation (Optional Enhancement)

Currently there's no explicit bolt count field. For stress calculations we need to know how
many bolts share the load. Two options:

**Option A (Recommended):** Assume a standard 4-bolt rectangular pattern. This matches the
existing `bolt_spacing` field (distance between bolt centers in one row). Document the
assumption explicitly.

**Option B:** Add a `bolt_count: u16` field to `ActuatorPlate`. This gives more flexibility
but adds another input field.

**Recommendation:** Start with Option A (4-bolt assumption) and add the field later if
needed. The validation crate can define this as a constant or derive it.

---

## Phase 2: Validation Logic (`crates/validation`)

### 2.0 Constants and Helpers

```rust
/// Safety factor applied to all force-based calculations.
/// Design force = SAFETY_FACTOR × expected force.
const SAFETY_FACTOR: u32 = 2;

/// Assumed number of mounting bolts in the bolt pattern.
const ASSUMED_BOLT_COUNT: u32 = 4;
```

All stress calculations use **integer arithmetic** where possible to stay `no_std` friendly.
Where division is needed, we'll use careful integer math to avoid floating-point:

```rust
// Instead of: stress = force / area (float division)
// Use: stress_mpa_x1000 = (force_n * 1000) / area_mm2
// Then compare: stress_mpa_x1000 <= yield_mpa * 1000
//
// Or more simply, rearrange to avoid division entirely:
// force_n <= yield_mpa * area_mm2  (since 1 MPa = 1 N/mm²)
```

The key insight: **1 MPa = 1 N/mm²**. This means we can often rearrange inequalities to
use only multiplication, avoiding floating-point entirely.

### 2.1 Force Input Validation

```rust
pub fn validate_expected_force(value: u32) -> Result<(), PlateValidationError> {
    if value == 0 {
        return Err(PlateValidationError::ExpectedForceTooSmall);
    }
    Ok(())
}
```

### 2.2 Pin Bearing Stress Check

When a pin transfers force to the plate, the contact area is `pin_diameter × plate_thickness`.

**Check:** `design_force ≤ yield_strength × pin_diameter × plate_thickness`

```
design_force_n = 2 × expected_force_per_pin
bearing_area_mm2 = pin_diameter_mm × plate_thickness_mm
allowable_force_n = yield_strength_mpa × bearing_area_mm2   // since 1 MPa = 1 N/mm²

PASS if: design_force_n ≤ allowable_force_n
```

This is pure integer multiplication — no floating point needed.

**Error:** `PinBearingStressExceeded { design_force_n: u32, allowable_force_n: u32 }`

### 2.3 Bolt Bearing Stress Check

Total reaction force is distributed across mounting bolts.

**Check:** `(total_design_force / bolt_count) ≤ yield_strength × bolt_diameter × plate_thickness`

```
total_design_force_n = 2 × expected_force_per_pin × pin_count
force_per_bolt_n = total_design_force_n / ASSUMED_BOLT_COUNT  // integer division (conservative: rounds down)
bolt_diameter_mm = bolt_size.nominal_diameter_mm()
bearing_area_mm2 = bolt_diameter_mm × plate_thickness_mm
allowable_force_n = yield_strength_mpa × bearing_area_mm2

PASS if: force_per_bolt_n ≤ allowable_force_n
```

Note: Integer division of total force by bolt count rounds down, which is *slightly*
non-conservative. To be safe, we round up: `(total + bolt_count - 1) / bolt_count`.

**Error:** `BoltBearingStressExceeded { force_per_bolt_n: u32, allowable_per_bolt_n: u32 }`

### 2.4 Plate Bending Stress Check

The plate spans between bolt rows under load from the actuator pins. Simplified model:
simply-supported beam of span `bolt_spacing`, loaded at center with total pin force.

Bending stress formula (rearranged to avoid division):
```
σ_bending = (3 × F × L) / (2 × w × t²)

Where:
  F = total design force = 2 × expected_force_per_pin × pin_count
  L = bolt_spacing (mm)
  w = bracket_width (mm)
  t = plate_thickness (mm)

Rearranged inequality (PASS when σ_bending ≤ σ_yield):
  3 × F × L ≤ 2 × σ_yield × w × t²
```

Both sides are integer expressions (all inputs are integers). We use `u64` intermediate
values to avoid overflow:

```rust
let lhs: u64 = 3 * (total_design_force as u64) * (bolt_spacing as u64);
let rhs: u64 = 2 * (yield_strength as u64) * (bracket_width as u64)
             * (plate_thickness as u64) * (plate_thickness as u64);

// PASS if lhs <= rhs
```

**Error:** `PlateBendingStressExceeded`

### 2.5 Bolt Edge Distance Check

Bolts need minimum material between the hole edge and the plate edge to prevent tearout.
Standard engineering practice: **edge distance ≥ 1.5 × bolt hole diameter**.

Since we assume a symmetric 4-bolt pattern:
```
available_edge_distance = (bracket_width - bolt_spacing) / 2   // mm, from bolt center to edge
required_edge_distance = 1.5 × clearance_hole_diameter         // mm

// Using integer math (multiply both sides by 2 to avoid the /2):
(bracket_width - bolt_spacing) ≥ 3 × clearance_hole_diameter
```

Since `clearance_hole_diameter` is currently `f32`, we can multiply by 10 and use integer
comparison, or switch the clearance values to be stored as tenths-of-mm (`u16`). The
simplest approach: use the nominal bolt diameter as a conservative proxy (it's always
smaller than the clearance hole).

```
Conservative check: (bracket_width - bolt_spacing) ≥ 3 × nominal_bolt_diameter
```

**Error:** `BoltEdgeDistanceTooSmall { available_mm: u16, required_mm: u16 }`

### 2.6 Pin Shear Tearout Check

The material between the pin hole edge and the nearest plate edge must resist shear tearout.
For pins arrayed along the bracket height:

```
available_edge = (bracket_height - (pin_count - 1) × pin_spacing) / 2
```

However, we don't currently have a `pin_spacing` field. For now, validate that the bracket
height can accommodate all pins with adequate edge clearance:

```
required_height = pin_count × pin_diameter × 3   // 1.5× diameter clearance on each side of each pin
PASS if: bracket_height ≥ required_height
```

This ensures at least 1× pin diameter spacing between pins and 1× pin diameter from
the outermost pin to the plate edge.

**Error:** `InsufficientPinClearance { bracket_height_mm: u16, required_mm: u16 }`

### 2.7 Minimum Thickness Advisory

After computing all stress checks, derive the minimum plate thickness that would satisfy
all bearing and bending constraints for the given material and force. This isn't a
pass/fail check — it provides user guidance:

```rust
pub fn minimum_thickness_mm(plate: &ActuatorPlate) -> u16 {
    // Solve for t from bearing: t ≥ design_force / (yield × pin_diameter)
    // Solve for t from bending: t² ≥ (3 × F × L) / (2 × yield × w)
    // Return the larger of the two
}
```

This function is useful for the frontend to suggest a minimum thickness when the user
changes material or force values.

### 2.8 Updated Error Enum

```rust
#[derive(Debug)]
pub enum PlateValidationError {
    // --- Existing (basic constraint) errors ---
    BoltSpacingTooSmall,
    BoltSizeInvalid,
    BracketHeightInvalid,
    BracketWidthInvalid,
    MaterialInvalid,
    PinDiameterInvalid,
    PinCountTooSmall,
    PinCountTooLarge,
    PlateThicknessInvalid,

    // --- New (force/stress) errors ---
    ExpectedForceTooSmall,
    PinBearingStressExceeded {
        design_force_n: u32,
        allowable_force_n: u32,
    },
    BoltBearingStressExceeded {
        force_per_bolt_n: u32,
        allowable_per_bolt_n: u32,
    },
    PlateBendingStressExceeded,
    BoltEdgeDistanceTooSmall {
        available_mm: u16,
        required_mm: u16,
    },
    InsufficientPinClearance {
        bracket_height_mm: u16,
        required_mm: u16,
    },
}
```

### 2.9 Updated `validate()` Function

The main `validate()` function runs checks in two phases:

1. **Basic constraints** (existing): non-zero, range checks — fast-fail if geometry is invalid
2. **Stress analysis** (new): material + force + thickness checks — only run if basic constraints pass

```rust
pub fn validate(plate: &ActuatorPlate) -> Result<(), PlateValidationError> {
    // Phase 1: Basic geometry constraints (existing)
    validate_bolt_spacing(plate.bolt_spacing.0)?;
    validate_bracket_height(plate.bracket_height.0)?;
    validate_bracket_width(plate.bracket_width.0)?;
    validate_pin_diameter(plate.pin_diameter.0)?;
    validate_pin_count(plate.pin_count)?;
    validate_plate_thickness(plate.plate_thickness.0)?;
    validate_expected_force(plate.expected_force_per_pin.0)?;

    // Phase 2: Stress analysis (new)
    validate_pin_bearing_stress(plate)?;
    validate_bolt_bearing_stress(plate)?;
    validate_plate_bending_stress(plate)?;
    validate_bolt_edge_distance(plate)?;
    validate_pin_clearance(plate)?;

    Ok(())
}
```

### 2.10 Collecting Multiple Errors (Future Enhancement)

Currently `validate()` returns on the first error. A future enhancement could collect all
errors and return them together:

```rust
pub fn validate_all(plate: &ActuatorPlate) -> Result<(), Vec<PlateValidationError>> { ... }
```

This is out of scope for the initial implementation but worth noting for the frontend UX.
The current single-error approach works fine for the WASM real-time validation pattern
(each field validated independently).

---

## Phase 3: WASM Bindings (`crates/validation/src/wasm.rs`)

### 3.1 New WASM Functions

Add bindings for the new force-related validators:

```rust
#[wasm_bindgen]
pub fn wasm_validate_expected_force(value: u32) -> Result<(), String> { ... }

/// Full stress analysis — takes all plate parameters including force.
/// Returns Ok(()) or Err(String) with the first stress violation found.
#[wasm_bindgen]
pub fn wasm_validate_stress(
    bolt_spacing: u16,
    bolt_size: &str,
    bracket_height: u16,
    bracket_width: u16,
    material: &str,
    pin_diameter: u16,
    pin_count: u16,
    plate_thickness: u16,
    expected_force_per_pin: u32,
) -> Result<(), String> { ... }

/// Returns the minimum recommended plate thickness in mm for the
/// given material, force, and geometry.
#[wasm_bindgen]
pub fn wasm_minimum_thickness(
    bolt_spacing: u16,
    bolt_size: &str,
    bracket_width: u16,
    material: &str,
    pin_diameter: u16,
    pin_count: u16,
    expected_force_per_pin: u32,
) -> u16 { ... }
```

The `wasm_validate_stress` function takes flat parameters (not a struct) because
`wasm-bindgen` doesn't support passing complex Rust structs directly. It constructs an
`ActuatorPlate` internally and runs the stress checks.

### 3.2 Error Message Format for Stress Errors

Stress errors include numeric context for the frontend to display:

```
"pin bearing stress exceeded: design force 2000 N exceeds allowable 1600 N for 8mm aluminum plate"
"bolt bearing stress exceeded: 1500 N per bolt exceeds allowable 1200 N"
"plate bending stress exceeded: plate too thin for the applied load"
"bolt edge distance too small: 8mm available, 15mm required"
"insufficient pin clearance: bracket height 40mm, need at least 90mm for 6 pins"
```

---

## Phase 4: Web API Updates (`crates/web`)

### 4.1 Request Type

The `ActuatorPlate` struct is already the request body for `/api/validate` and
`/api/generate`. Adding the new field automatically updates both endpoints.

### 4.2 Enhanced Validation Response (Optional)

Consider adding a stress summary to successful validation responses:

```rust
#[derive(Serialize, ToSchema)]
struct ValidationSuccessResponse {
    valid: bool,
    message: String,
    // New: stress analysis results
    stress_summary: Option<StressSummary>,
}

#[derive(Serialize, ToSchema)]
struct StressSummary {
    safety_factor: f32,            // Always 2.0
    pin_bearing_utilization: f32,  // 0.0–1.0 (design_force / allowable)
    bolt_bearing_utilization: f32,
    bending_utilization: f32,
    minimum_thickness_mm: u16,
}
```

Utilization ratios help users understand how close they are to the limits — a plate at
0.95 utilization is technically valid but has very little margin.

### 4.3 OpenAPI Updates

- Add `Newtons` to schema components
- Add `StressSummary` to schema components
- Update `ActuatorPlate` schema with the new field
- Update example values

---

## Phase 5: Frontend Updates (`frontend/`)

### 5.1 New Input Field

Add an "Expected Force per Pin" numeric input to the form, in the Pins field group:

```
Pins
├── Pin Diameter (mm)     [existing]
├── Pin Count             [existing]
└── Force per Pin (N)     [new]
```

### 5.2 WASM Validation Integration

Add TypeScript wrapper:

```typescript
export async function validateExpectedForce(value: number): Promise<ValidationResult> {
  await initValidation();
  return validate(() => wasm_validate_expected_force(value));
}
```

### 5.3 Stress Feedback UI

After all fields are valid, run `wasm_validate_stress()` to check the combined
stress analysis. Display results as:

- **Green**: Utilization < 60% — "Plate design is well within safety margins"
- **Yellow**: Utilization 60–85% — "Plate design is adequate"
- **Red**: Utilization > 85% — "Consider increasing thickness or changing material"

Show the minimum recommended thickness as a hint when the plate fails stress checks.

### 5.4 Updated API Call

```typescript
const response = await fetch('/api/generate', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({
    bolt_spacing: 60,
    bolt_size: "M10",
    bracket_height: 400,
    bracket_width: 300,
    material: "aluminum",
    pin_diameter: 10,
    pin_count: 6,
    plate_thickness: 8,
    expected_force_per_pin: 500,  // NEW
  }),
});
```

---

## Phase 6: Testing Strategy

### 6.1 Material Property Tests (domain crate)

- Verify each material returns correct yield, tensile, shear, modulus, density values
- Ensure all values are > 0
- Verify aluminum has lowest density, brass has lowest yield strength

### 6.2 Individual Stress Check Unit Tests (validation crate)

For each stress check, test:

1. **Passing case**: Thick steel plate with small forces
2. **Failing case**: Thin brass plate with large forces
3. **Boundary case**: Force exactly at the allowable limit
4. **Safety factor verification**: A plate that would pass at 1× safety but fails at 2×

Example test matrix for pin bearing:

| Material | Pin Ø (mm) | Thickness (mm) | Force (N) | Design Force (N) | Allowable (N) | Result |
|---|---|---|---|---|---|---|
| Aluminum | 10 | 8 | 500 | 1000 | 276 × 80 = 22,080 | PASS |
| Brass | 5 | 3 | 300 | 600 | 124 × 15 = 1,860 | PASS |
| Brass | 3 | 2 | 200 | 400 | 124 × 6 = 744 | PASS |
| Brass | 3 | 2 | 500 | 1000 | 124 × 6 = 744 | FAIL |

### 6.3 Integrated Validation Tests

- Default plate configuration passes all checks
- Plates with extreme forces fail appropriately
- Changing material from steel to brass (with same geometry/force) can flip pass → fail
- Increasing thickness can fix a previously failing plate

### 6.4 Regression Tests

- All 20 existing validation unit tests continue to pass
- All 6 existing API integration tests continue to pass (after updating request payloads)
- Existing frontend behavior preserved for basic constraint checks

### 6.5 Integer Overflow Tests

Verify that stress calculations don't overflow with maximum input values:

```
Max u16 values: 65535
Max force: u32::MAX = 4,294,967,295

Worst case multiplication:
3 × total_design_force × bolt_spacing  (in bending check)
= 3 × (2 × 4,294,967,295 × 65535) × 65535
This WILL overflow u64. Need to validate input ranges or use u128.
```

**Important**: Add reasonable upper bounds on input values to prevent overflow:
- `expected_force_per_pin` ≤ 100,000 N (100 kN — far beyond typical actuator needs)
- Existing dimension fields already capped at 65,535 mm by `u16`

With these bounds, worst case bending calculation:
```
3 × (2 × 100,000 × 12) × 65,535 = 3 × 2,400,000 × 65,535 = 471,852,000,000
```
Fits in `u64` (max 1.8 × 10¹⁹). Safe.

---

## Implementation Order

Suggested order to minimize breakage and allow incremental testing:

1. **Domain crate**: Add `Newtons`, material properties, update `ActuatorPlate`
2. **Validation crate**: Add `validate_expected_force` + update `validate()`
3. **Tests**: Make existing tests pass with the new field (add default force)
4. **Validation crate**: Add stress check functions one at a time, with tests
5. **WASM bindings**: Add new functions
6. **Web crate**: Update handlers, add stress summary response
7. **Frontend**: Add force input, stress feedback UI
8. **Integration tests**: End-to-end validation with force

---

## Design Decisions Summary

| Decision | Choice | Rationale |
|---|---|---|
| Safety factor | Hardcoded 2× | User requirement; simpler than configurable |
| Bolt count | Assumed 4 | Matches standard rectangular pattern; avoids new field |
| Arithmetic | Integer (no float) | `no_std` friendly; exact comparisons; rearrange inequalities |
| Overflow protection | `u64` intermediates + input bounds | Prevents UB without requiring `u128` |
| Error reporting | Single error (first fail) | Matches existing pattern; collect-all is future work |
| Force units | Newtons (`u32`) | SI standard; `u32` gives range to 4.3 MN |
| Stress unit consistency | All in MPa (= N/mm²) | Avoids unit conversion in calculations |

---

## Constraints and Risks

1. **`no_std` compatibility**: All new code must compile without `std`. Integer arithmetic
   helps here, but we need to be careful with the `Display` impl for new error variants
   (avoid `format!` which requires `alloc`).

2. **WASM binary size**: Adding material properties and stress calculations will increase
   the WASM binary. Expected impact is minimal (a few KB) since it's all simple arithmetic.

3. **Breaking API change**: Adding `expected_force_per_pin` to `ActuatorPlate` is a
   breaking change. Options:
   - Make it `Option<Newtons>` with `#[serde(default)]` for backward compatibility
   - Accept the breaking change (this is pre-1.0 software)
   - **Recommendation**: Use `#[serde(default)]` with a sensible default so existing API
     clients don't break, but stress checks are skipped when force is 0/None.

4. **Simplified bending model**: The simply-supported beam model is conservative but
   simplified. Real plates have 2D stress distributions. This is acceptable for a
   configuration tool — it prevents obviously bad designs without replacing FEA.

5. **Cache invalidation**: Adding a new field to `ActuatorPlate` changes the cache key
   hash. Existing cached results become unreachable (not invalid, just orphaned).
   This is acceptable.
