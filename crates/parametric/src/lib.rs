use std::path::{Path, PathBuf};

use domain::ActuatorPlate;
use kittycad::types::{ApiCallStatus, FileExportFormat, FileImportFormat};
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
         export materialColor = \"{}\"\n\
         export pinDiameter = {}\n\
         export pinCount = {}",
        plate.plate_thickness.0,
        bolt_hole_diameter,
        plate.bolt_spacing.0,
        plate.bracket_height.0,
        plate.bracket_width.0,
        plate.material.as_hex_code(),
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

/// Create a kittycad client from environment variables.
/// Requires KITTYCAD_API_TOKEN or ZOO_API_TOKEN to be set.
fn create_zoo_client() -> kittycad::Client {
    kittycad::Client::new_from_env()
}

pub async fn generate_model(plate: &ActuatorPlate) -> Result<GenerationResult, AllErrors> {
    if let Err(e) = validation::validate(plate) {
        let msg = format!("Validation failed: {}", e);
        tracing::error!("{}", msg);
        return Err(AllErrors::ValidationError(msg));
    }

    // Create a temporary directory for this generation request
    let temp_dir = match TempDir::new() {
        Ok(dir) => dir,
        Err(e) => {
            let msg = format!("Failed to create temp directory: {}", e);
            tracing::error!("{}", msg);
            return Err(AllErrors::GeneratorError(msg));
        }
    };

    let temp_path = temp_dir.path();

    // Copy KCL source files to temp dir
    if let Err(e) = copy_kcl_sources(temp_path) {
        let msg = format!(
            "Failed to copy KCL sources from {}: {}",
            get_kcl_source_dir(),
            e
        );
        tracing::error!("{}", msg);
        return Err(AllErrors::GeneratorError(msg));
    }

    // Write params.kcl to temp dir
    if let Err(e) = write_params_file(plate, temp_path) {
        let msg = format!("Failed to write params file: {}", e);
        tracing::error!("{}", msg);
        return Err(AllErrors::GeneratorError(msg));
    }

    // Generate STEP file using zoo CLI
    // Note: KCL execution requires the Zoo modeling WebSocket API, which is not
    // exposed as a REST endpoint in the kittycad crate. The zoo CLI handles the
    // WebSocket session internally via kcl-lib.
    if let Err(e) = generate_step_in_dir(plate, temp_path) {
        let msg = format!("Failed to generate STEP file: {:?}", e);
        tracing::error!("{}", msg);
        return Err(AllErrors::GeneratorError(msg));
    }

    // Generate glTF file using the kittycad API (STEP → glTF conversion)
    if let Err(e) = generate_gltf_via_api(temp_path).await {
        let msg = format!("Failed to generate glTF file: {:?}", e);
        tracing::error!("{}", msg);
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

/// Generate STEP file in the specified directory using the zoo CLI.
///
/// KCL execution requires the Zoo modeling WebSocket API, which is handled
/// by the zoo CLI internally. The kittycad REST crate does not expose a
/// KCL execution endpoint.
fn generate_step_in_dir(plate: &ActuatorPlate, dir: &Path) -> Result<(), ValidationError> {
    if let Err(e) = validation::validate(plate) {
        tracing::error!("Validation failed: {}", e);
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
        Ok(stat) if stat.success() => Ok(()),
        Ok(stat) => {
            tracing::error!("zoo kcl export exited with status: {}", stat);
            Err(ValidationError::NoStep)
        }
        Err(e) => {
            tracing::error!("Failed to run zoo CLI: {}", e);
            Err(ValidationError::NoStep)
        }
    }
}

/// Generate glTF file by converting the STEP file using the Zoo API via the kittycad crate.
///
/// This replaces the previous approach of shelling out to `zoo file convert`.
/// Uses the kittycad crate's file conversion endpoint (POST /file/conversion/{src_format}/{output_format})
/// for a cleaner, more reliable conversion with proper error handling.
async fn generate_gltf_via_api(dir: &Path) -> Result<(), AllErrors> {
    let step_file = dir.join("output.step");

    if !step_file.exists() {
        return Err(AllErrors::GeneratorError(
            "STEP file does not exist for glTF conversion".to_string(),
        ));
    }

    // Read the STEP file contents
    let step_bytes = tokio::fs::read(&step_file).await.map_err(|e| {
        AllErrors::GeneratorError(format!("Failed to read STEP file: {}", e))
    })?;

    let client = create_zoo_client();

    // Convert STEP to glTF using the kittycad file conversion API
    let conversion = client
        .file()
        .create_conversion(
            FileExportFormat::Gltf,
            FileImportFormat::Step,
            &bytes::Bytes::from(step_bytes),
        )
        .await
        .map_err(|e| {
            AllErrors::GeneratorError(format!("Zoo API file conversion failed: {}", e))
        })?;

    // Handle async conversion - poll until complete if needed
    let outputs = match conversion.status {
        ApiCallStatus::Completed => conversion.outputs,
        ApiCallStatus::Failed => {
            let error_msg = conversion
                .error
                .unwrap_or_else(|| "Unknown conversion error".to_string());
            return Err(AllErrors::GeneratorError(format!(
                "Zoo API conversion failed: {}",
                error_msg
            )));
        }
        // For async operations, poll until completion
        status => {
            tracing::info!(
                "Conversion is {:?}, polling for completion (id: {})",
                status,
                conversion.id
            );
            poll_conversion_completion(&client, conversion.id).await?
        }
    };

    // Extract and write the glTF output file
    let outputs = outputs.ok_or_else(|| {
        AllErrors::GeneratorError("No output files returned from conversion".to_string())
    })?;

    // Find the glTF file in the outputs
    let gltf_data = outputs
        .iter()
        .find(|(path, _)| path.ends_with(".gltf") || path.ends_with(".glb"))
        .map(|(_, data)| data)
        .ok_or_else(|| {
            let available_keys: Vec<&String> = outputs.keys().collect();
            AllErrors::GeneratorError(format!(
                "No glTF file found in conversion outputs. Available: {:?}",
                available_keys
            ))
        })?;

    // Write glTF file to the output directory
    let gltf_path = dir.join("source.gltf");
    tokio::fs::write(&gltf_path, &gltf_data.0).await.map_err(|e| {
        AllErrors::GeneratorError(format!("Failed to write glTF file: {}", e))
    })?;

    tracing::info!("Generated glTF via Zoo API at {:?}", gltf_path);
    Ok(())
}

/// Poll the Zoo API for completion of an async file conversion operation.
async fn poll_conversion_completion(
    client: &kittycad::Client,
    operation_id: uuid::Uuid,
) -> Result<Option<std::collections::HashMap<String, kittycad::types::base64::Base64Data>>, AllErrors>
{
    let max_attempts = 60; // Poll for up to 5 minutes (60 * 5s)
    let poll_interval = std::time::Duration::from_secs(5);

    for attempt in 1..=max_attempts {
        tokio::time::sleep(poll_interval).await;

        let result = client
            .api_calls()
            .get_async_operation(operation_id)
            .await
            .map_err(|e| {
                AllErrors::GeneratorError(format!("Failed to poll conversion status: {}", e))
            })?;

        match result {
            kittycad::types::AsyncApiCallOutput::FileConversion {
                status,
                outputs,
                error,
                ..
            } => match status {
                ApiCallStatus::Completed => {
                    tracing::info!(
                        "Conversion completed after {} poll attempts",
                        attempt
                    );
                    return Ok(outputs);
                }
                ApiCallStatus::Failed => {
                    let error_msg =
                        error.unwrap_or_else(|| "Unknown conversion error".to_string());
                    return Err(AllErrors::GeneratorError(format!(
                        "Zoo API conversion failed: {}",
                        error_msg
                    )));
                }
                _ => {
                    tracing::debug!(
                        "Conversion still in progress (attempt {}/{})",
                        attempt,
                        max_attempts
                    );
                }
            },
            other => {
                return Err(AllErrors::GeneratorError(format!(
                    "Unexpected async operation type: {:?}",
                    std::mem::discriminant(&other)
                )));
            }
        }
    }

    Err(AllErrors::GeneratorError(format!(
        "Conversion timed out after {} attempts",
        max_attempts
    )))
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

    #[tokio::test]
    #[ignore]
    async fn test_generate_model_succeeds_with_valid_plate() {
        let plate = ActuatorPlate::default();

        // This test requires:
        // 1. zoo CLI to be installed and authenticated
        // 2. KITTYCAD_API_TOKEN or ZOO_API_TOKEN environment variable set
        let result = generate_model(&plate).await;

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

        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(generate_model(&plate));

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
        // Verify material is included (default is aluminum)
        assert!(content.contains("export materialColor = \"#A9ACB6\""));
        assert!(content.contains("export pinDiameter"));
        assert!(content.contains("export pinCount = 6"));

        // Temp directory is automatically cleaned up
    }

    #[test]
    fn test_generate_step_fails_with_missing_kcl_files() {
        let plate = ActuatorPlate::default();
        let temp_dir = TempDir::new().unwrap();

        // Don't copy KCL source files - this should cause zoo to fail
        write_params_file(&plate, temp_dir.path()).unwrap();

        let result = generate_step_in_dir(&plate, temp_dir.path());
        // Should fail because main.kcl doesn't exist in the temp dir
        assert!(result.is_err());
    }

    // This test requires the `zoo` CLI to be installed and for the user to be authenticated; it is ignored by default
    #[tokio::test]
    #[ignore]
    async fn test_generate_step_creates_file_with_zoo_cli() {
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
            Ok(()) => {
                assert!(temp_dir.path().join("output.step").exists());
            }
            Err(e) => {
                panic!("Failed to run zoo command: {:?}. Is zoo CLI installed? Is the user authenticated?", e);
            }
        }

        // Temp directory is automatically cleaned up
    }

    // This test requires the `zoo` CLI, KITTYCAD_API_TOKEN, and user authentication
    #[tokio::test]
    #[ignore]
    async fn test_generate_gltf_via_api() {
        let plate = ActuatorPlate::default();
        let temp_dir = TempDir::new().unwrap();

        // Copy KCL sources and write params
        copy_kcl_sources(temp_dir.path()).unwrap();
        write_params_file(&plate, temp_dir.path()).unwrap();

        // Generate STEP file first
        let step_result = generate_step_in_dir(&plate, temp_dir.path());
        assert!(step_result.is_ok(), "STEP generation should succeed");

        // Convert STEP to glTF via the Zoo API
        let result = generate_gltf_via_api(temp_dir.path()).await;

        match result {
            Ok(()) => {
                assert!(temp_dir.path().join("source.gltf").exists());
            }
            Err(e) => {
                panic!("Failed to convert via Zoo API: {:?}. Is KITTYCAD_API_TOKEN set?", e);
            }
        }

        // Temp directory is automatically cleaned up
    }
}
