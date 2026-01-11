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
         export const pinDiameter = {:?}",
        plate.plate_thickness,
        plate.bolt_diameter,
        plate.bolt_spacing,
        plate.bracket_height,
        plate.bracket_width,
        plate.pin_diameter
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

    Ok(())
}

pub fn generate_step(plate: ActuatorPlate) -> Result<ExitStatus, ValidationError> {
    if let Err(e) = validation::validate(&plate) {
        eprintln!("oops: {}", e);
        return Err(ValidationError::NoStep);
    }

    let a = std::process::Command::new("zoo")
        .args(&[
            "kcl",
            "export",
            "--output-format=step",
            "main.kcl", // TODO: create this main.kcl
            "output_dir",
        ])
        .status();

    match a {
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
    fn it_fails_when_it_should() {
        let mut plate = ActuatorPlate::default();
        plate.bolt_diameter = Millimeters(0);
        let err = Err(ValidationError::NoStep);
        assert_eq!(err, generate_step(plate))
    }
}
