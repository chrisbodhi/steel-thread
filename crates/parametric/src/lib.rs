use std::path::{Path, PathBuf};
use std::process::ExitStatus;

use domain::ActuatorPlate;
use tempfile::TempDir;

pub trait Validation {
    // TODO: figure out how to mesh `plate` arg here with generic trait
    // TODO: We may want a T that matches ValidationError when we define this trait for real
    fn is_valid(plate: ActuatorPlate) -> Result<(), ValidationError>;
}

#[derive(Debug, PartialEq)]
pub enum ValidationError {
    NoStep,
}

#[derive(Debug, PartialEq)]
pub enum GeneratorError {
    CliError,
}

#[derive(Debug)]
pub enum AllErrors {
    GeneratorError(String),
    ValidationError(String),
}

/// Result of a successful model generation, containing paths to generated files.
/// The TempDir is held to prevent cleanup until the caller is done with the files.
#[derive(Debug)]
pub struct GenerationResult {
    /// The temporary directory containing all generated files.
    /// Files are cleaned up when this is dropped.
    pub temp_dir: TempDir,
    /// Path to the generated STEP file
    pub step_file: PathBuf,
    /// Path to the generated glTF file
    pub gltf_file: PathBuf,
}

/// Get the source directory containing KCL files.
/// Checks KCL_SRC_DIR environment variable first, then falls back to local paths.
fn get_kcl_source_dir() -> String {
    // First check environment variable (for production deployment)
    if let Ok(env_dir) = std::env::var("KCL_SRC_DIR") {
        return env_dir;
    }

    // Fall back to local development paths
    if Path::new("crates/parametric/src/main.kcl").exists() {
        "crates/parametric/src".to_string()
    } else {
        "src".to_string()
    }
}

/// Write params.kcl to the specified directory
fn write_params_file(plate: &ActuatorPlate, dir: &Path) -> std::io::Result<()> {
    // Use clearance hole diameter for mounting bolts
    let bolt_hole_diameter = plate.bolt_size.clearance_hole_diameter_mm();

    let content = format!(
        "@settings(defaultLengthUnit = mm, kclVersion = 1.0)\n\n\
         export plateThickness = {}\n\
         export boltDiameter = {}\n\
         export boltSpacing = {}\n\
         export bracketHeight = {}\n\
         export bracketWidth = {}\n\
         export pinDiameter = {}\n\
         export pinCount = {}",
        plate.plate_thickness.0,
        bolt_hole_diameter,
        plate.bolt_spacing.0,
        plate.bracket_height.0,
        plate.bracket_width.0,
        plate.pin_diameter.0,
        plate.pin_count
    );

    std::fs::write(dir.join("params.kcl"), content)?;
    Ok(())
}

/// Copy KCL source files to the temp directory
fn copy_kcl_sources(temp_dir: &Path) -> std::io::Result<()> {
    let source_dir = get_kcl_source_dir();

    // Copy main.kcl and plate.kcl to temp dir
    std::fs::copy(
        Path::new(&source_dir).join("main.kcl"),
        temp_dir.join("main.kcl"),
    )?;
    std::fs::copy(
        Path::new(&source_dir).join("plate.kcl"),
        temp_dir.join("plate.kcl"),
    )?;

    Ok(())
}

pub fn generate_model(plate: &ActuatorPlate) -> Result<GenerationResult, AllErrors> {
    if let Err(e) = validation::validate(plate) {
        let msg = format!("Validation failed: {}", e);
        eprintln!("{}", msg);
        return Err(AllErrors::ValidationError(msg));
    }

    // Create a temporary directory for this generation request
    let temp_dir = match TempDir::new() {
        Ok(dir) => dir,
        Err(e) => {
            let msg = format!("Failed to create temp directory: {}", e);
            eprintln!("{}", msg);
            return Err(AllErrors::GeneratorError(msg));
        }
    };

    let temp_path = temp_dir.path();

    // Copy KCL source files to temp dir
    if let Err(e) = copy_kcl_sources(temp_path) {
        let msg = format!("Failed to copy KCL sources from {}: {}", get_kcl_source_dir(), e);
        eprintln!("{}", msg);
        return Err(AllErrors::GeneratorError(msg));
    }

    // Write params.kcl to temp dir
    if let Err(e) = write_params_file(plate, temp_path) {
        let msg = format!("Failed to write params file: {}", e);
        eprintln!("{}", msg);
        return Err(AllErrors::GeneratorError(msg));
    }

    // Generate STEP file
    if let Err(e) = generate_step_in_dir(plate, temp_path) {
        let msg = format!("Failed to generate STEP file: {:?}", e);
        eprintln!("{}", msg);
        return Err(AllErrors::GeneratorError(msg));
    }

    // Generate glTF file
    if let Err(e) = generate_gltf_in_dir(plate, temp_path) {
        let msg = format!("Failed to generate glTF file: {:?}", e);
        eprintln!("{}", msg);
        return Err(AllErrors::GeneratorError(msg));
    }

    let step_file = temp_path.join("output.step");
    let gltf_file = temp_path.join("source.gltf");

    Ok(GenerationResult {
        temp_dir,
        step_file,
        gltf_file,
    })
}

/// Generate STEP file in the specified directory
fn generate_step_in_dir(plate: &ActuatorPlate, dir: &Path) -> Result<ExitStatus, ValidationError> {
    if let Err(e) = validation::validate(plate) {
        eprintln!("oops: {}", e);
        return Err(ValidationError::NoStep);
    }

    let main_kcl = dir.join("main.kcl");

    let status = std::process::Command::new("zoo")
        .args([
            "kcl",
            "export",
            "--output-format=step",
            main_kcl.to_str().unwrap(),
            dir.to_str().unwrap(),
        ])
        .status();

    match status {
        Ok(stat) => Ok(stat),
        Err(e) => {
            eprintln!("ouch: {}", e);
            Err(ValidationError::NoStep)
        }
    }
}

/// Generate glTF file in the specified directory by converting the STEP file
fn generate_gltf_in_dir(plate: &ActuatorPlate, dir: &Path) -> Result<ExitStatus, ValidationError> {
    if let Err(e) = validation::validate(plate) {
        eprintln!("oops: {}", e);
        return Err(ValidationError::NoStep);
    }

    let step_file = dir.join("output.step");

    // Check if STEP file exists
    if !step_file.exists() {
        eprintln!("STEP file does not exist at {:?}", step_file);
        return Err(ValidationError::NoStep);
    }

    // Convert STEP file to glTF using zoo file convert
    let status = std::process::Command::new("zoo")
        .args([
            "file",
            "convert",
            "--src-format=step",
            "--output-format=gltf",
            step_file.to_str().unwrap(),
            dir.to_str().unwrap(),
        ])
        .status();

    match status {
        Ok(stat) => Ok(stat),
        Err(e) => {
            eprintln!("ouch: {}", e);
            Err(ValidationError::NoStep)
        }
    }
}

#[cfg(test)]
mod tests {
    use domain::Millimeters;

    use super::*;

    #[test]
    fn test_generate_step_fails_with_invalid_plate() {
        let mut plate = ActuatorPlate::default();
        plate.bolt_spacing = Millimeters(0); // Invalid bolt spacing

        let temp_dir = TempDir::new().unwrap();
        let result = generate_step_in_dir(&plate, temp_dir.path());

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ValidationError::NoStep);
    }

    #[test]
    #[ignore]
    fn test_generate_model_succeeds_with_valid_plate() {
        let plate = ActuatorPlate::default();

        // This test requires zoo CLI to be installed and authenticated
        // It will generate params.kcl, STEP, and glTF files in a temp directory
        let result = generate_model(&plate);

        // Should succeed in generating all files
        assert!(result.is_ok());

        let gen_result = result.unwrap();
        assert!(gen_result.step_file.exists());
        assert!(gen_result.gltf_file.exists());

        // Temp directory and files are automatically cleaned up when gen_result is dropped
    }

    #[test]
    fn test_generate_model_fails_with_invalid_plate() {
        let mut plate = ActuatorPlate::default();
        plate.bolt_spacing = Millimeters(0);

        let result = generate_model(&plate);

        assert!(result.is_err());
        match result.unwrap_err() {
            AllErrors::ValidationError(msg) => {
                assert!(msg.contains("Validation failed"));
            }
            _ => panic!("Expected ValidationError"),
        }
    }

    #[test]
    fn test_write_params_file_creates_valid_kcl() {
        let plate = ActuatorPlate::default();
        let temp_dir = TempDir::new().unwrap();

        let result = write_params_file(&plate, temp_dir.path());
        assert!(result.is_ok());

        let params_path = temp_dir.path().join("params.kcl");

        // Read and verify the file content
        let content = std::fs::read_to_string(&params_path).unwrap();

        // Check for correct format
        assert!(content.starts_with("@settings(defaultLengthUnit = mm, kclVersion = 1.0)"));
        assert!(content.contains("export plateThickness"));
        assert!(content.contains("export boltDiameter"));
        // Verify that bolt diameter uses clearance hole size (M10 = 11.0mm clearance)
        assert!(content.contains("export boltDiameter = 11"));
        assert!(content.contains("export boltSpacing"));
        assert!(content.contains("export bracketHeight"));
        assert!(content.contains("export bracketWidth"));
        assert!(content.contains("export pinDiameter"));
        assert!(content.contains("export pinCount = 6"));

        // Temp directory is automatically cleaned up
    }

    #[test]
    fn test_generate_gltf_fails_with_invalid_plate() {
        let mut plate = ActuatorPlate::default();
        plate.plate_thickness = Millimeters(0); // Invalid plate thickness

        let temp_dir = TempDir::new().unwrap();
        let result = generate_gltf_in_dir(&plate, temp_dir.path());

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ValidationError::NoStep);
    }

    // This test requires the `zoo` CLI to be installed and for the user to be authenticated; it is ignored by default
    #[test]
    #[ignore]
    fn test_generate_step_creates_file_with_zoo_cli() {
        let plate = ActuatorPlate::default();
        let temp_dir = TempDir::new().unwrap();

        // Copy KCL sources and write params
        copy_kcl_sources(temp_dir.path()).unwrap();
        write_params_file(&plate, temp_dir.path()).unwrap();

        // This will only pass if, as pre-requisites:
        // 1. zoo CLI is installed
        // 2. user is authenticated against zoo
        let result = generate_step_in_dir(&plate, temp_dir.path());

        match result {
            Ok(status) => {
                // Check if command succeeded
                assert!(status.success(), "zoo command should succeed");
                assert!(temp_dir.path().join("output.step").exists());
            }
            Err(e) => {
                // If zoo is not installed, the test should be skipped
                panic!("Failed to run zoo command: {:?}. Is zoo CLI installed? Is the user authenticated?", e);
            }
        }

        // Temp directory is automatically cleaned up
    }

    // This test requires the `zoo` CLI to be installed and for the user to be authenticated; it is ignored by default
    #[test]
    #[ignore]
    fn test_generate_gltf_creates_file_with_zoo_cli() {
        let plate = ActuatorPlate::default();
        let temp_dir = TempDir::new().unwrap();

        // Copy KCL sources and write params
        copy_kcl_sources(temp_dir.path()).unwrap();
        write_params_file(&plate, temp_dir.path()).unwrap();

        // Generate STEP file first (glTF generation now converts from STEP)
        let step_result = generate_step_in_dir(&plate, temp_dir.path());
        assert!(step_result.is_ok(), "STEP generation should succeed");
        assert!(
            step_result.unwrap().success(),
            "STEP generation should succeed"
        );

        let result = generate_gltf_in_dir(&plate, temp_dir.path());

        match result {
            Ok(status) => {
                // Check if command succeeded
                assert!(status.success(), "zoo command should succeed");
                assert!(temp_dir.path().join("source.gltf").exists());
            }
            Err(e) => {
                // If zoo is not installed, the test should be skipped
                panic!("Failed to run zoo command: {:?}. Is zoo CLI installed? Is the user authenticated?", e);
            }
        }

        // Temp directory is automatically cleaned up
    }
}
