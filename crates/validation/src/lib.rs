#![no_std]

use domain::ActuatorPlate;

// WebAssembly bindings (only compiled for wasm32 target)
#[cfg(target_arch = "wasm32")]
pub mod wasm;

// TODO: move into just-actuator-only file

// TODO: make a trait that works for items besides plates
// ---this will allow us to accept a Validator trait in other
// crates, eg crates/parameteric
// pub trait Validation {
//     fn is_valid() -> Result<(), ValidationError>;
// }
// pub enum ValidationError {}

pub fn validate(plate: &ActuatorPlate) -> Result<(), PlateValidationError> {
    validate_bolt_spacing(plate.bolt_spacing.0)?;
    // bolt_size is validated by the type system (enum only allows valid ISO sizes)
    validate_bracket_height(plate.bracket_height.0)?;
    validate_bracket_width(plate.bracket_width.0)?;
    // material is validated by the type system (enum only allows valid materials)
    validate_pin_diameter(plate.pin_diameter.0)?;
    validate_pin_count(plate.pin_count)?;
    validate_plate_thickness(plate.plate_thickness.0)?;
    Ok(())
}

pub fn validate_bolt_spacing(value: u16) -> Result<(), PlateValidationError> {
    if value == 0 {
        return Err(PlateValidationError::BoltSpacingTooSmall);
    }
    Ok(())
}

/// Validate that a bolt size string is a valid ISO metric size.
///
/// Accepts standard metric bolt designations: "M3", "M4", "M5", "M6", "M8", "M10", "M12"
/// (case-insensitive).
pub fn validate_bolt_size(value: &str) -> Result<(), PlateValidationError> {
    let upper = value.trim();
    match upper {
        "M3" | "m3" | "M4" | "m4" | "M5" | "m5" | "M6" | "m6" | "M8" | "m8" | "M10" | "m10"
        | "M12" | "m12" => Ok(()),
        _ => Err(PlateValidationError::BoltSizeInvalid),
    }
}

/// Validate that a material string is a valid material type.
///
/// Accepts: "aluminum", "stainless_steel", "carbon_steel", "brass" (case-insensitive).
pub fn validate_material(value: &str) -> Result<(), PlateValidationError> {
    let trimmed = value.trim();
    // Case-insensitive comparison using manual matching
    match trimmed {
        "aluminum" | "Aluminum" | "ALUMINUM" | "stainless_steel" | "Stainless_Steel"
        | "STAINLESS_STEEL" | "stainlessSteel" | "StainlessSteel" | "carbon_steel"
        | "Carbon_Steel" | "CARBON_STEEL" | "carbonSteel" | "CarbonSteel" | "brass" | "Brass"
        | "BRASS" => Ok(()),
        _ => Err(PlateValidationError::MaterialInvalid),
    }
}

pub fn validate_bracket_height(value: u16) -> Result<(), PlateValidationError> {
    if value == 0 {
        return Err(PlateValidationError::BracketHeightInvalid);
    }
    Ok(())
}

pub fn validate_bracket_width(value: u16) -> Result<(), PlateValidationError> {
    if value == 0 {
        return Err(PlateValidationError::BracketWidthInvalid);
    }
    Ok(())
}

pub fn validate_pin_diameter(value: u16) -> Result<(), PlateValidationError> {
    if value == 0 {
        return Err(PlateValidationError::PinDiameterInvalid);
    }
    Ok(())
}

pub fn validate_pin_count(value: u16) -> Result<(), PlateValidationError> {
    if value == 0 {
        return Err(PlateValidationError::PinCountTooSmall);
    }
    if value > 12 {
        return Err(PlateValidationError::PinCountTooLarge);
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
    BoltSizeInvalid,
    BracketHeightInvalid,
    BracketWidthInvalid,
    MaterialInvalid,
    PinDiameterInvalid,
    PinCountTooSmall,
    PinCountTooLarge,
    PlateThicknessInvalid,
}

impl core::fmt::Display for PlateValidationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::BoltSpacingTooSmall => write!(f, "bolt spacing must be greater than 0"),
            Self::BoltSizeInvalid => write!(
                f,
                "bolt size must be a standard ISO metric size: M3, M4, M5, M6, M8, M10, or M12"
            ),
            Self::BracketHeightInvalid => write!(f, "bracket height must be greater than 0"),
            Self::BracketWidthInvalid => write!(f, "bracket width must be greater than 0"),
            Self::MaterialInvalid => write!(
                f,
                "material must be one of: aluminum, stainless_steel, carbon_steel, or brass"
            ),
            Self::PinDiameterInvalid => write!(f, "pin diameter must be greater than 0"),
            Self::PinCountTooSmall => write!(f, "pin count must be at least 1"),
            Self::PinCountTooLarge => write!(f, "pin count must not exceed 12"),
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
    use domain::{BoltSize, Material, Millimeters};

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
    fn test_validate_bolt_size_valid() {
        assert!(validate_bolt_size("M3").is_ok());
        assert!(validate_bolt_size("M4").is_ok());
        assert!(validate_bolt_size("M5").is_ok());
        assert!(validate_bolt_size("M6").is_ok());
        assert!(validate_bolt_size("M8").is_ok());
        assert!(validate_bolt_size("M10").is_ok());
        assert!(validate_bolt_size("M12").is_ok());
        // Test case-insensitive
        assert!(validate_bolt_size("m3").is_ok());
        assert!(validate_bolt_size("m10").is_ok());
    }

    #[test]
    fn test_validate_bolt_size_invalid() {
        assert!(validate_bolt_size("M7").is_err());
        assert!(validate_bolt_size("M11").is_err());
        assert!(validate_bolt_size("M2").is_err());
        assert!(validate_bolt_size("10").is_err());
        assert!(validate_bolt_size("").is_err());
        assert!(validate_bolt_size("invalid").is_err());

        let result = validate_bolt_size("M7");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            PlateValidationError::BoltSizeInvalid
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
    fn test_validate_bracket_width_valid() {
        assert!(validate_bracket_width(30).is_ok());
    }

    #[test]
    fn test_validate_bracket_width_invalid() {
        let result = validate_bracket_width(0);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            PlateValidationError::BracketWidthInvalid
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
    fn test_validate_pin_count_valid() {
        assert!(validate_pin_count(1).is_ok());
        assert!(validate_pin_count(6).is_ok());
        assert!(validate_pin_count(12).is_ok());
    }

    #[test]
    fn test_validate_pin_count_too_small() {
        let result = validate_pin_count(0);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            PlateValidationError::PinCountTooSmall
        ));
    }

    #[test]
    fn test_validate_pin_count_too_large() {
        let result = validate_pin_count(13);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            PlateValidationError::PinCountTooLarge
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
            bolt_size: BoltSize::M10,
            bracket_height: Millimeters(40),
            bracket_width: Millimeters(30),
            material: Material::Aluminum,
            pin_diameter: Millimeters(10),
            pin_count: 6,
            plate_thickness: Millimeters(8),
        };
        assert!(validate(&plate).is_ok());
    }

    #[test]
    fn test_validate_full_plate_invalid_bolt_spacing() {
        let plate = ActuatorPlate {
            bolt_spacing: Millimeters(0),
            bolt_size: BoltSize::M10,
            bracket_height: Millimeters(40),
            bracket_width: Millimeters(30),
            material: Material::Aluminum,
            pin_diameter: Millimeters(10),
            pin_count: 6,
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
    fn test_validate_material_valid() {
        assert!(validate_material("aluminum").is_ok());
        assert!(validate_material("Aluminum").is_ok());
        assert!(validate_material("ALUMINUM").is_ok());
        assert!(validate_material("stainless_steel").is_ok());
        assert!(validate_material("Stainless_Steel").is_ok());
        assert!(validate_material("carbon_steel").is_ok());
        assert!(validate_material("brass").is_ok());
        assert!(validate_material("Brass").is_ok());
    }

    #[test]
    fn test_validate_material_invalid() {
        assert!(validate_material("wood").is_err());
        assert!(validate_material("plastic").is_err());
        assert!(validate_material("").is_err());
        assert!(validate_material("titanium").is_err());

        let result = validate_material("invalid");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            PlateValidationError::MaterialInvalid
        ));
    }

    #[test]
    fn test_error_display_messages() {
        assert_eq!(
            PlateValidationError::BoltSpacingTooSmall.to_string(),
            "bolt spacing must be greater than 0"
        );
        assert_eq!(
            PlateValidationError::BoltSizeInvalid.to_string(),
            "bolt size must be a standard ISO metric size: M3, M4, M5, M6, M8, M10, or M12"
        );
        assert_eq!(
            PlateValidationError::BracketHeightInvalid.to_string(),
            "bracket height must be greater than 0"
        );
        assert_eq!(
            PlateValidationError::BracketWidthInvalid.to_string(),
            "bracket width must be greater than 0"
        );
        assert_eq!(
            PlateValidationError::MaterialInvalid.to_string(),
            "material must be one of: aluminum, stainless_steel, carbon_steel, or brass"
        );
        assert_eq!(
            PlateValidationError::PinDiameterInvalid.to_string(),
            "pin diameter must be greater than 0"
        );
        assert_eq!(
            PlateValidationError::PinCountTooSmall.to_string(),
            "pin count must be at least 1"
        );
        assert_eq!(
            PlateValidationError::PinCountTooLarge.to_string(),
            "pin count must not exceed 12"
        );
        assert_eq!(
            PlateValidationError::PlateThicknessInvalid.to_string(),
            "plate thickness must be greater than 0"
        );
    }
}
