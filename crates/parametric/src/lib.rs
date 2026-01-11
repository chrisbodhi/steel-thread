use std::process::ExitStatus;

use domain::ActuatorPlate;
use validation;

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

#[derive(Debug, PartialEq)]
pub enum AllErrors {
    GeneratorError,
    ValidationError,
}

fn generate_params_file(plate: &ActuatorPlate) -> std::io::Result<()> {
    let content = format!(
        "@settings(defaultLengthUnit = mm, kclVersion = 1.0)\n\n\
         export const plateThickness = {:?}\n\
         export const boltDiameter = {:?}\n\
         export const boltSpacing = {:?}\n\
         export const bracketHeight = {:?}\n\
         export const bracketWidth = {:?}\n\
         export const pinDiameter = {:?}\n\
         export const pinCount = {}",
        plate.plate_thickness,
        plate.bolt_diameter,
        plate.bolt_spacing,
        plate.bracket_height,
        plate.bracket_width,
        plate.pin_diameter,
        plate.pin_count
    );
    std::fs::write("params.kcl", content)?;

    Ok(())
}

pub fn generate_model(plate: &ActuatorPlate) -> Result<(), AllErrors> {
    if let Err(e) = validation::validate(&plate) {
        eprintln!("oops: {}", e);
        return Err(AllErrors::ValidationError);
    }

    if let Err(e) = generate_params_file(&plate) {
        eprintln!("oops on that param: {}", e);
        return Err(AllErrors::GeneratorError);
    }

    // Generate STEP file
    if let Err(e) = generate_step(plate.clone()) {
        eprintln!("oops generating STEP: {:?}", e);
        return Err(AllErrors::GeneratorError);
    }

    // Generate glTF file
    if let Err(e) = generate_gltf(plate.clone()) {
        eprintln!("oops generating glTF: {:?}", e);
        return Err(AllErrors::GeneratorError);
    }

    Ok(())
}

pub fn generate_step(plate: ActuatorPlate) -> Result<ExitStatus, ValidationError> {
    if let Err(e) = validation::validate(&plate) {
        eprintln!("oops: {}", e);
        return Err(ValidationError::NoStep);
    }

    let status = std::process::Command::new("zoo")
        .args(&[
            "kcl",
            "export",
            "--output-format=step",
            "src/main.kcl",
            "output_dir",
        ])
        .status();

    match status {
        Ok(stat) => Ok(stat),
        Err(e) => {
            eprintln!("ouch: {}", e);
            return Err(ValidationError::NoStep);
        }
    }
}

pub fn generate_gltf(plate: ActuatorPlate) -> Result<ExitStatus, ValidationError> {
    if let Err(e) = validation::validate(&plate) {
        eprintln!("oops: {}", e);
        return Err(ValidationError::NoStep);
    }

    let status = std::process::Command::new("zoo")
        .args(&[
            "kcl",
            "export",
            "--output-format=gltf",
            "src/main.kcl",
            "output_dir",
        ])
        .status();

    match status {
        Ok(stat) => Ok(stat),
        Err(e) => {
            eprintln!("ouch: {}", e);
            return Err(ValidationError::NoStep);
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
        plate.bolt_diameter = Millimeters(0);

        let result = generate_step(plate);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ValidationError::NoStep);
    }

    #[test]
    #[ignore]
    fn test_generate_model_succeeds_with_valid_plate() {
        let plate = ActuatorPlate::default();

        // This test requires zoo CLI to be installed and authenticated
        // It will generate params.kcl, STEP, and glTF files
        let result = generate_model(&plate);

        // Should succeed in generating all files
        assert!(result.is_ok());

        // Cleanup (params.kcl may have been consumed by zoo)
        std::fs::remove_file("params.kcl").ok();
        std::fs::remove_file("output_dir/output.step").ok();
        std::fs::remove_file("output_dir/output.gltf").ok();
    }

    #[test]
    fn test_generate_model_fails_with_invalid_plate() {
        let mut plate = ActuatorPlate::default();
        plate.bolt_spacing = Millimeters(0);

        let result = generate_model(&plate);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), AllErrors::ValidationError);
    }

    #[test]
    fn test_generate_params_file_creates_valid_kcl() {
        let plate = ActuatorPlate::default();

        let result = generate_params_file(&plate);
        assert!(result.is_ok());

        // Read and verify the file content
        let content = std::fs::read_to_string("params.kcl").unwrap();

        // Check for correct format
        assert!(content.starts_with("@settings(defaultLengthUnit = mm, kclVersion = 1.0)"));
        assert!(content.contains("export const plateThickness"));
        assert!(content.contains("export const boltDiameter"));
        assert!(content.contains("export const boltSpacing"));
        assert!(content.contains("export const bracketHeight"));
        assert!(content.contains("export const bracketWidth"));
        assert!(content.contains("export const pinDiameter"));
        assert!(content.contains("export const pinCount = 6"));

        // Cleanup
        std::fs::remove_file("params.kcl").ok();
    }

    #[test]
    fn test_generate_gltf_fails_with_invalid_plate() {
        let mut plate = ActuatorPlate::default();
        plate.bolt_diameter = Millimeters(0);

        let result = generate_gltf(plate);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ValidationError::NoStep);
    }

    // This test requires the `zoo` CLI to be installed and for the user to be authenticated; it is ignored by default
    #[test]
    #[ignore]
    fn test_generate_step_creates_file_with_zoo_cli() {
        let plate = ActuatorPlate::default();

        // This will only pass if, as pre-requisites:
        // 1. zoo CLI is installed
        // 2. user is authenticated against zoo
        // 3. main.kcl exists
        // 4. output_dir exists
        let result = generate_step(plate);

        match result {
            Ok(status) => {
                // Check if command succeeded
                assert!(status.success(), "zoo command should succeed");
            }
            Err(e) => {
                // If zoo is not installed, the test should be skipped
                panic!("Failed to run zoo command: {:?}. Is zoo CLI installed? Is the user authenticated?", e);
            }
        }

        // Cleanup
        std::fs::remove_file("params.kcl").ok();
        std::fs::remove_file("output_dir/output.step").ok();
    }

    // This test requires the `zoo` CLI to be installed and for the user to be authenticated; it is ignored by default
    #[test]
    #[ignore]
    fn test_generate_gltf_creates_file_with_zoo_cli() {
        let plate = ActuatorPlate::default();

        // This will only pass if, as pre-requisites:
        // 1. zoo CLI is installed
        // 2. user is authenticated against zoo
        // 3. main.kcl exists
        // 4. output_dir exists
        let result = generate_gltf(plate);

        match result {
            Ok(status) => {
                // Check if command succeeded
                assert!(status.success(), "zoo command should succeed");
            }
            Err(e) => {
                // If zoo is not installed, the test should be skipped
                panic!("Failed to run zoo command: {:?}. Is zoo CLI installed? Is the user authenticated?", e);
            }
        }

        // Cleanup
        std::fs::remove_file("params.kcl").ok();
        std::fs::remove_file("output_dir/output.gltf").ok();
    }
}
