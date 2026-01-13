# File Locations and CLI Call Documentation

## Current File Layout

### Input Files (Source Code)
- `crates/parametric/src/main.kcl` - Main KCL program that imports params and plate
- `crates/parametric/src/plate.kcl` - Plate geometry definition
- `crates/parametric/src/params.kcl` - **GENERATED** - Parameter values (bolt size, pin count, etc.)

### Output Files (Generated)
- `output_dir/output.step` - STEP 3D model file
- `output_dir/source.gltf` - glTF 3D model file (converted from STEP)

## Working Directory Assumptions

**Current assumption**: Code runs from project root (`/Users/b/code/bas/steel-thread`)

### Path Resolution Logic

The code uses a fallback pattern to handle both:
1. Running from project root (production, tests)
2. Running from `crates/parametric/` (development)

```rust
// For main.kcl
let kcl_path = if std::path::Path::new("crates/parametric/src/main.kcl").exists() {
    "crates/parametric/src/main.kcl"  // Project root
} else {
    "src/main.kcl"                     // From crates/parametric/
};

// For params.kcl (same logic)
let params_path = if std::path::Path::new("crates/parametric/src/main.kcl").exists() {
    "crates/parametric/src/params.kcl"
} else {
    "src/params.kcl"
};
```

## Zoo CLI Commands

### Command 1: Generate STEP from KCL
```bash
zoo kcl export \
  --output-format=step \
  crates/parametric/src/main.kcl \
  output_dir
```

**Inputs:**
- `crates/parametric/src/main.kcl` (relative to working directory)
- `crates/parametric/src/params.kcl` (imported by main.kcl)
- `crates/parametric/src/plate.kcl` (imported by main.kcl)

**Output:**
- `output_dir/output.step`

**Requirements:**
- Working directory must be project root OR paths must be adjusted
- params.kcl MUST be in same directory as main.kcl (relative import)

### Command 2: Convert STEP to glTF
```bash
zoo file convert \
  --src-format=step \
  --output-format=gltf \
  output_dir/output.step \
  output_dir
```

**Input:**
- `output_dir/output.step`

**Output:**
- `output_dir/source.gltf`

**Requirements:**
- STEP file must exist before conversion
- Works from any working directory (uses absolute/explicit paths)

## Critical Dependencies

### params.kcl Location
**CRITICAL**: params.kcl MUST be in same directory as main.kcl because:
```kcl
// main.kcl line 3:
import plateThickness, boltDiameter, boltSpacing, bracketHeight, bracketWidth, pinDiameter, pinCount from "params.kcl"
```

The import is **relative to main.kcl's location**, not the working directory.

### params.kcl Format
```kcl
@settings(defaultLengthUnit = mm, kclVersion = 1.0)

export plateThickness = 8
export boltDiameter = 10
export boltSpacing = 60
export bracketHeight = 400
export bracketWidth = 300
export pinDiameter = 10
export pinCount = 8
```

**Note:**
- No `const` keyword (deprecated in KCL)
- Numeric values only (not `Millimeters(8)`)

## AWS Deployment Concerns

### Issues to Address

1. **Hardcoded Relative Paths**
   - Current: Assumes running from project root
   - Problem: AWS Lambda/ECS working directory may differ
   - Solution: Use absolute paths or environment variables

2. **Shared Output Directory**
   - Current: All requests write to `output_dir/output.step` and `output_dir/source.gltf`
   - Problem: Concurrent requests will overwrite each other's files
   - Solution: Use unique output directories per request (e.g., `output_dir/{request_id}/`)

3. **File Cleanup**
   - Current: No cleanup of generated files
   - Problem: Disk space accumulation
   - Solution: Clean up after serving files to client, or use temp directories

4. **Working Directory Assumptions**
   - Current: Code assumes it knows where files are relative to cwd
   - Problem: Container/Lambda may set cwd differently
   - Solution:
     - Set explicit working directory in deployment config
     - OR use absolute paths based on binary location
     - OR use environment variables for all paths

### Recommended Changes for AWS

```rust
// Use environment variables or config
let kcl_src_dir = env::var("KCL_SRC_DIR")
    .unwrap_or_else(|_| "/app/crates/parametric/src".to_string());
let output_base_dir = env::var("OUTPUT_DIR")
    .unwrap_or_else(|_| "/tmp/output".to_string());

// Per-request output directory
let request_id = uuid::Uuid::new_v4();
let output_dir = format!("{}/{}", output_base_dir, request_id);
std::fs::create_dir_all(&output_dir)?;

// Absolute paths
let main_kcl = format!("{}/main.kcl", kcl_src_dir);
let params_kcl = format!("{}/params.kcl", kcl_src_dir);
let step_file = format!("{}/output.step", output_dir);
let gltf_file = format!("{}/source.gltf", output_dir);
```

### Temporary Directory Option

```rust
use std::env::temp_dir;

// Create temp directory for this request
let temp_dir = temp_dir().join(format!("actuator-{}", uuid::Uuid::new_v4()));
std::fs::create_dir_all(&temp_dir)?;

// Copy KCL source files to temp (or use symlinks)
// Generate params.kcl in temp
// Run zoo CLI in temp
// Clean up temp dir after serving response
```

## Current Flow

```
1. API receives plate parameters
   ↓
2. generate_params_file() writes:
   crates/parametric/src/params.kcl
   ↓
3. generate_step() calls zoo:
   zoo kcl export --output-format=step \
     crates/parametric/src/main.kcl \
     output_dir
   → Creates: output_dir/output.step
   ↓
4. generate_gltf() calls zoo:
   zoo file convert --src-format=step --output-format=gltf \
     output_dir/output.step \
     output_dir
   → Creates: output_dir/source.gltf
   ↓
5. API serves files to client
```

## Testing Checklist

- [ ] Verify working directory in AWS deployment
- [ ] Test concurrent requests don't overwrite each other
- [ ] Verify file cleanup works
- [ ] Test absolute vs relative path handling
- [ ] Verify zoo CLI works in container environment
- [ ] Test file permissions in deployment environment
