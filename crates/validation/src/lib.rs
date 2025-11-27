#![no_std]

use domain::ActuatorPlate;

// TODO: move into just-actuator-only file

pub fn validate(plate: &ActuatorPlate) -> Result<(), PlateValidationError> {
    // Validate each field
    if plate.bolt_spacing.0 == 0 {
        return Err(PlateValidationError::BoltSpacingTooSmall);
    }
    if plate.bolt_diameter.0 == 0 {
        return Err(PlateValidationError::BoltDiameterInvalid);
    }
    if plate.bracket_height.0 == 0 {
        return Err(PlateValidationError::BracketHeightInvalid);
    }
    if plate.pin_diameter.0 == 0 {
        return Err(PlateValidationError::PinDiameterInvalid);
    }
    if plate.plate_thickness.0 == 0 {
        return Err(PlateValidationError::PlateThicknessInvalid);
    }

    Ok(())
}

#[derive(Debug)]
pub enum PlateValidationError {
    BoltSpacingTooSmall,
    BoltDiameterInvalid,
    BracketHeightInvalid,
    PinDiameterInvalid,
    PlateThicknessInvalid,
}

impl core::fmt::Display for PlateValidationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::BoltSpacingTooSmall => write!(f, "bolt spacing must be greater than 0"),
            Self::BoltDiameterInvalid => write!(f, "bolt diameter must be greater than 0"),
            Self::BracketHeightInvalid => write!(f, "bracket height must be greater than 0"),
            Self::PinDiameterInvalid => write!(f, "pin diameter must be greater than 0"),
            Self::PlateThicknessInvalid => write!(f, "plate thickness must be greater than 0"),
        }
    }
}

impl core::error::Error for PlateValidationError {}
