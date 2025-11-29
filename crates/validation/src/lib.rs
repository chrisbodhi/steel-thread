#![no_std]

use domain::ActuatorPlate;

// TODO: move into just-actuator-only file

pub fn validate(plate: &ActuatorPlate) -> Result<(), PlateValidationError> {
    validate_bolt_spacing(plate.bolt_spacing.0)?;
    validate_bolt_diameter(plate.bolt_diameter.0)?;
    validate_bracket_height(plate.bracket_height.0)?;
    validate_pin_diameter(plate.pin_diameter.0)?;
    validate_plate_thickness(plate.plate_thickness.0)?;
    Ok(())
}

pub fn validate_bolt_spacing(value: u16) -> Result<(), PlateValidationError> {
    if value == 0 {
        return Err(PlateValidationError::BoltSpacingTooSmall);
    }
    Ok(())
}

pub fn validate_bolt_diameter(value: u16) -> Result<(), PlateValidationError> {
    if value == 0 {
        return Err(PlateValidationError::BoltDiameterInvalid);
    }
    Ok(())
}

pub fn validate_bracket_height(value: u16) -> Result<(), PlateValidationError> {
    if value == 0 {
        return Err(PlateValidationError::BracketHeightInvalid);
    }
    Ok(())
}

pub fn validate_pin_diameter(value: u16) -> Result<(), PlateValidationError> {
    if value == 0 {
        return Err(PlateValidationError::PinDiameterInvalid);
    }
    Ok(())
}

pub fn validate_plate_thickness(value: u16) -> Result<(), PlateValidationError> {
    if value == 0 {
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

#[cfg(test)]
mod tests {
    extern crate alloc;
    use alloc::string::ToString;

    use super::*;
    use domain::Millimeters;

    #[test]
    fn test_validate_bolt_spacing_valid() {
        assert!(validate_bolt_spacing(60).is_ok());
        assert!(validate_bolt_spacing(1).is_ok());
        assert!(validate_bolt_spacing(u16::MAX).is_ok());
    }

    #[test]
    fn test_validate_bolt_spacing_invalid() {
        let result = validate_bolt_spacing(0);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            PlateValidationError::BoltSpacingTooSmall
        ));
    }

    #[test]
    fn test_validate_bolt_diameter_valid() {
        assert!(validate_bolt_diameter(10).is_ok());
    }

    #[test]
    fn test_validate_bolt_diameter_invalid() {
        let result = validate_bolt_diameter(0);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            PlateValidationError::BoltDiameterInvalid
        ));
    }

    #[test]
    fn test_validate_bracket_height_valid() {
        assert!(validate_bracket_height(40).is_ok());
    }

    #[test]
    fn test_validate_bracket_height_invalid() {
        let result = validate_bracket_height(0);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            PlateValidationError::BracketHeightInvalid
        ));
    }

    #[test]
    fn test_validate_pin_diameter_valid() {
        assert!(validate_pin_diameter(10).is_ok());
    }

    #[test]
    fn test_validate_pin_diameter_invalid() {
        let result = validate_pin_diameter(0);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            PlateValidationError::PinDiameterInvalid
        ));
    }

    #[test]
    fn test_validate_plate_thickness_valid() {
        assert!(validate_plate_thickness(8).is_ok());
    }

    #[test]
    fn test_validate_plate_thickness_invalid() {
        let result = validate_plate_thickness(0);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            PlateValidationError::PlateThicknessInvalid
        ));
    }

    #[test]
    fn test_validate_full_plate_valid() {
        let plate = ActuatorPlate {
            bolt_spacing: Millimeters(60),
            bolt_diameter: Millimeters(10),
            bracket_height: Millimeters(40),
            pin_diameter: Millimeters(10),
            plate_thickness: Millimeters(8),
        };
        assert!(validate(&plate).is_ok());
    }

    #[test]
    fn test_validate_full_plate_invalid_bolt_spacing() {
        let plate = ActuatorPlate {
            bolt_spacing: Millimeters(0),
            bolt_diameter: Millimeters(10),
            bracket_height: Millimeters(40),
            pin_diameter: Millimeters(10),
            plate_thickness: Millimeters(8),
        };
        let result = validate(&plate);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            PlateValidationError::BoltSpacingTooSmall
        ));
    }

    #[test]
    fn test_error_display_messages() {
        assert_eq!(
            PlateValidationError::BoltSpacingTooSmall.to_string(),
            "bolt spacing must be greater than 0"
        );
        assert_eq!(
            PlateValidationError::BoltDiameterInvalid.to_string(),
            "bolt diameter must be greater than 0"
        );
        assert_eq!(
            PlateValidationError::BracketHeightInvalid.to_string(),
            "bracket height must be greater than 0"
        );
        assert_eq!(
            PlateValidationError::PinDiameterInvalid.to_string(),
            "pin diameter must be greater than 0"
        );
        assert_eq!(
            PlateValidationError::PlateThicknessInvalid.to_string(),
            "plate thickness must be greater than 0"
        );
    }
}
