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

/// Safety factor applied to all force-based calculations.
/// Design force = SAFETY_FACTOR × expected force.
const SAFETY_FACTOR: u32 = 2;

/// Assumed number of mounting bolts in a standard rectangular pattern.
const ASSUMED_BOLT_COUNT: u32 = 4;

pub fn validate(plate: &ActuatorPlate) -> Result<(), PlateValidationError> {
    // Phase 1: Basic geometry constraints
    validate_bolt_spacing(plate.bolt_spacing.0)?;
    // bolt_size is validated by the type system (enum only allows valid ISO sizes)
    validate_bracket_height(plate.bracket_height.0)?;
    validate_bracket_width(plate.bracket_width.0)?;
    // material is validated by the type system (enum only allows valid materials)
    validate_pin_diameter(plate.pin_diameter.0)?;
    validate_pin_count(plate.pin_count)?;
    validate_plate_thickness(plate.plate_thickness.0)?;
    validate_expected_force(plate.expected_force_per_pin.0)?;

    // Phase 2: Stress analysis (material + force + thickness)
    validate_pin_bearing_stress(plate)?;
    validate_bolt_bearing_stress(plate)?;
    validate_plate_bending_stress(plate)?;
    validate_bolt_edge_distance(plate)?;
    validate_pin_clearance(plate)?;

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

/// Maximum allowed force per pin (100 kN). Prevents u64 overflow in stress calculations.
const MAX_FORCE_PER_PIN: u32 = 100_000;

pub fn validate_expected_force(value: u32) -> Result<(), PlateValidationError> {
    if value == 0 {
        return Err(PlateValidationError::ExpectedForceTooSmall);
    }
    if value > MAX_FORCE_PER_PIN {
        return Err(PlateValidationError::ExpectedForceTooLarge);
    }
    Ok(())
}

/// Check that the plate won't crush at pin holes under the design load.
///
/// Bearing stress = force / (pin_diameter × plate_thickness).
/// Rearranged to integer math: design_force ≤ yield_strength × pin_diameter × thickness.
/// Since 1 MPa = 1 N/mm², this is a direct comparison.
pub fn validate_pin_bearing_stress(plate: &ActuatorPlate) -> Result<(), PlateValidationError> {
    let design_force = (plate.expected_force_per_pin.0 as u64) * (SAFETY_FACTOR as u64);
    let yield_mpa = plate.material.yield_strength_mpa() as u64;
    let pin_d = plate.pin_diameter.0 as u64;
    let thickness = plate.plate_thickness.0 as u64;

    let allowable = yield_mpa * pin_d * thickness;

    if design_force > allowable {
        return Err(PlateValidationError::PinBearingStressExceeded {
            design_force_n: design_force as u32,
            allowable_force_n: allowable as u32,
        });
    }
    Ok(())
}

/// Check that the plate won't crush at bolt holes under the total design load.
///
/// Total force from all pins is distributed across ASSUMED_BOLT_COUNT bolts.
/// Uses ceiling division for conservative force-per-bolt.
pub fn validate_bolt_bearing_stress(plate: &ActuatorPlate) -> Result<(), PlateValidationError> {
    let total_design_force = (plate.expected_force_per_pin.0 as u64)
        * (SAFETY_FACTOR as u64)
        * (plate.pin_count as u64);

    // Ceiling division: (total + bolt_count - 1) / bolt_count
    let bolt_count = ASSUMED_BOLT_COUNT as u64;
    let force_per_bolt = total_design_force.div_ceil(bolt_count);

    let yield_mpa = plate.material.yield_strength_mpa() as u64;
    let bolt_d = plate.bolt_size.nominal_diameter_mm() as u64;
    let thickness = plate.plate_thickness.0 as u64;

    let allowable = yield_mpa * bolt_d * thickness;

    if force_per_bolt > allowable {
        return Err(PlateValidationError::BoltBearingStressExceeded {
            force_per_bolt_n: force_per_bolt as u32,
            allowable_per_bolt_n: allowable as u32,
        });
    }
    Ok(())
}

/// Check that the plate won't yield in bending between bolt rows.
///
/// Simplified model: simply-supported beam of span bolt_spacing, loaded at center.
/// σ_bending = (3 × F × L) / (2 × w × t²)
/// Rearranged: 3 × F × L ≤ 2 × σ_yield × w × t²
pub fn validate_plate_bending_stress(plate: &ActuatorPlate) -> Result<(), PlateValidationError> {
    let total_design_force = (plate.expected_force_per_pin.0 as u64)
        * (SAFETY_FACTOR as u64)
        * (plate.pin_count as u64);
    let span = plate.bolt_spacing.0 as u64;

    let lhs: u64 = 3 * total_design_force * span;

    let yield_mpa = plate.material.yield_strength_mpa() as u64;
    let width = plate.bracket_width.0 as u64;
    let thickness = plate.plate_thickness.0 as u64;

    let rhs: u64 = 2 * yield_mpa * width * thickness * thickness;

    if lhs > rhs {
        return Err(PlateValidationError::PlateBendingStressExceeded);
    }
    Ok(())
}

/// Check that bolts have adequate edge distance to prevent tearout.
///
/// Standard practice: edge distance ≥ 1.5 × bolt hole diameter.
/// Conservative check using nominal bolt diameter (smaller than clearance hole):
/// (bracket_width - bolt_spacing) ≥ 3 × nominal_bolt_diameter
pub fn validate_bolt_edge_distance(plate: &ActuatorPlate) -> Result<(), PlateValidationError> {
    let width = plate.bracket_width.0;
    let spacing = plate.bolt_spacing.0;

    if width <= spacing {
        return Err(PlateValidationError::BoltEdgeDistanceTooSmall {
            available_mm: 0,
            required_mm: plate.bolt_size.nominal_diameter_mm() * 3,
        });
    }

    let available = width - spacing;
    let required = plate.bolt_size.nominal_diameter_mm() * 3;

    if available < required {
        return Err(PlateValidationError::BoltEdgeDistanceTooSmall {
            available_mm: available,
            required_mm: required,
        });
    }
    Ok(())
}

/// Check that the bracket height can accommodate all pins with adequate clearance.
///
/// Each pin needs 3× its diameter of vertical space (1.5× clearance on each side).
/// required_height = pin_count × pin_diameter × 3
pub fn validate_pin_clearance(plate: &ActuatorPlate) -> Result<(), PlateValidationError> {
    let required = (plate.pin_count as u32) * (plate.pin_diameter.0 as u32) * 3;

    if (plate.bracket_height.0 as u32) < required {
        return Err(PlateValidationError::InsufficientPinClearance {
            bracket_height_mm: plate.bracket_height.0,
            required_mm: required as u16,
        });
    }
    Ok(())
}

/// Stress utilization ratios (0.0–1.0+). Values > 1.0 indicate failure.
pub struct StressUtilization {
    pub pin_bearing: f32,
    pub bolt_bearing: f32,
    pub bending: f32,
}

/// Compute stress utilization ratios for a valid plate configuration.
/// Each ratio is design_load / allowable_load (0.0 = no load, 1.0 = at limit).
pub fn stress_utilization(plate: &ActuatorPlate) -> StressUtilization {
    let design_force = (plate.expected_force_per_pin.0 as f32) * (SAFETY_FACTOR as f32);
    let yield_mpa = plate.material.yield_strength_mpa() as f32;
    let pin_d = plate.pin_diameter.0 as f32;
    let thickness = plate.plate_thickness.0 as f32;
    let bolt_d = plate.bolt_size.nominal_diameter_mm() as f32;
    let pin_count = plate.pin_count as f32;
    let bolt_count = ASSUMED_BOLT_COUNT as f32;
    let span = plate.bolt_spacing.0 as f32;
    let width = plate.bracket_width.0 as f32;

    let pin_allowable = yield_mpa * pin_d * thickness;
    let pin_bearing = if pin_allowable > 0.0 {
        design_force / pin_allowable
    } else {
        1.0
    };

    let total_design = design_force * pin_count;
    let force_per_bolt = total_design / bolt_count;
    let bolt_allowable = yield_mpa * bolt_d * thickness;
    let bolt_bearing = if bolt_allowable > 0.0 {
        force_per_bolt / bolt_allowable
    } else {
        1.0
    };

    // bending: σ = (3*F*L) / (2*w*t²), utilization = σ / σ_yield
    let bending_stress = if width * thickness * thickness > 0.0 {
        (3.0 * total_design * span) / (2.0 * width * thickness * thickness)
    } else {
        yield_mpa // assume failure
    };
    let bending = if yield_mpa > 0.0 {
        bending_stress / yield_mpa
    } else {
        1.0
    };

    StressUtilization {
        pin_bearing,
        bolt_bearing,
        bending,
    }
}

/// Returns the minimum plate thickness (mm) that satisfies bearing and bending
/// constraints for the given material, geometry, and force. Useful for UI guidance.
pub fn minimum_thickness_mm(plate: &ActuatorPlate) -> u16 {
    let design_force = (plate.expected_force_per_pin.0 as u64) * (SAFETY_FACTOR as u64);
    let yield_mpa = plate.material.yield_strength_mpa() as u64;
    let pin_d = plate.pin_diameter.0 as u64;

    // From bearing: t ≥ design_force / (yield × pin_diameter)
    // Ceiling division
    let t_bearing = if yield_mpa * pin_d > 0 {
        design_force.div_ceil(yield_mpa * pin_d)
    } else {
        1
    };

    // From bending: t² ≥ (3 × F_total × L) / (2 × yield × w)
    let total_design_force = design_force * (plate.pin_count as u64);
    let span = plate.bolt_spacing.0 as u64;
    let width = plate.bracket_width.0 as u64;

    let numerator = 3 * total_design_force * span;
    let denominator = 2 * yield_mpa * width;

    let t_bending_sq = if denominator > 0 {
        numerator.div_ceil(denominator)
    } else {
        1
    };

    // Integer square root (ceiling): find smallest t where t*t >= t_bending_sq
    let mut t_bending: u64 = 1;
    while t_bending * t_bending < t_bending_sq {
        t_bending += 1;
    }

    let min = if t_bearing > t_bending {
        t_bearing
    } else {
        t_bending
    };

    // Clamp to at least 1mm
    if min < 1 { 1 } else { min as u16 }
}

#[derive(Debug)]
pub enum PlateValidationError {
    // Basic constraint errors
    BoltSpacingTooSmall,
    BoltSizeInvalid,
    BracketHeightInvalid,
    BracketWidthInvalid,
    MaterialInvalid,
    PinDiameterInvalid,
    PinCountTooSmall,
    PinCountTooLarge,
    PlateThicknessInvalid,

    // Force/stress errors
    ExpectedForceTooSmall,
    ExpectedForceTooLarge,
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
            Self::ExpectedForceTooSmall => {
                write!(f, "expected force per pin must be greater than 0")
            }
            Self::ExpectedForceTooLarge => {
                write!(f, "expected force per pin must not exceed 100,000 N")
            }
            Self::PinBearingStressExceeded {
                design_force_n,
                allowable_force_n,
            } => write!(
                f,
                "pin bearing stress exceeded: design force {} N exceeds allowable {} N",
                design_force_n, allowable_force_n
            ),
            Self::BoltBearingStressExceeded {
                force_per_bolt_n,
                allowable_per_bolt_n,
            } => write!(
                f,
                "bolt bearing stress exceeded: {} N per bolt exceeds allowable {} N",
                force_per_bolt_n, allowable_per_bolt_n
            ),
            Self::PlateBendingStressExceeded => {
                write!(f, "plate bending stress exceeded: plate too thin for the applied load")
            }
            Self::BoltEdgeDistanceTooSmall {
                available_mm,
                required_mm,
            } => write!(
                f,
                "bolt edge distance too small: {} mm available, {} mm required",
                available_mm, required_mm
            ),
            Self::InsufficientPinClearance {
                bracket_height_mm,
                required_mm,
            } => write!(
                f,
                "insufficient pin clearance: bracket height {} mm, need at least {} mm",
                bracket_height_mm, required_mm
            ),
        }
    }
}

impl core::error::Error for PlateValidationError {}

#[cfg(test)]
mod tests {
    extern crate alloc;
    use alloc::string::ToString;

    use super::*;
    use domain::{BoltSize, Material, Millimeters, Newtons};

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

    // Helper: a structurally sound plate that passes all checks
    fn valid_plate() -> ActuatorPlate {
        ActuatorPlate {
            bolt_spacing: Millimeters(60),
            bolt_size: BoltSize::M10,
            bracket_height: Millimeters(200),
            bracket_width: Millimeters(100),
            material: Material::Aluminum,
            pin_diameter: Millimeters(10),
            pin_count: 4,
            plate_thickness: Millimeters(10),
            expected_force_per_pin: Newtons(500),
        }
    }

    #[test]
    fn test_validate_full_plate_valid() {
        assert!(validate(&valid_plate()).is_ok());
    }

    #[test]
    fn test_validate_full_plate_invalid_bolt_spacing() {
        let mut plate = valid_plate();
        plate.bolt_spacing = Millimeters(0);
        let result = validate(&plate);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            PlateValidationError::BoltSpacingTooSmall
        ));
    }

    // --- Force validation ---

    #[test]
    fn test_validate_expected_force_valid() {
        assert!(validate_expected_force(1).is_ok());
        assert!(validate_expected_force(500).is_ok());
    }

    #[test]
    fn test_validate_expected_force_zero() {
        assert!(matches!(
            validate_expected_force(0).unwrap_err(),
            PlateValidationError::ExpectedForceTooSmall
        ));
    }

    #[test]
    fn test_full_plate_zero_force_rejected() {
        let mut plate = valid_plate();
        plate.expected_force_per_pin = Newtons(0);
        assert!(matches!(
            validate(&plate).unwrap_err(),
            PlateValidationError::ExpectedForceTooSmall
        ));
    }

    // --- Pin bearing stress ---

    #[test]
    fn test_pin_bearing_pass() {
        // Aluminum, 10mm pin, 10mm thick: allowable = 276 * 10 * 10 = 27,600 N
        // Design force = 500 * 2 = 1,000 N. Well under limit.
        assert!(validate_pin_bearing_stress(&valid_plate()).is_ok());
    }

    #[test]
    fn test_pin_bearing_fail_thin_brass() {
        let mut plate = valid_plate();
        plate.material = Material::Brass; // yield 124 MPa
        plate.pin_diameter = Millimeters(3);
        plate.plate_thickness = Millimeters(2);
        plate.expected_force_per_pin = Newtons(500);
        // allowable = 124 * 3 * 2 = 744 N, design = 1000 N → fail
        let result = validate_pin_bearing_stress(&plate);
        assert!(result.is_err());
        match result.unwrap_err() {
            PlateValidationError::PinBearingStressExceeded {
                design_force_n,
                allowable_force_n,
            } => {
                assert_eq!(design_force_n, 1000);
                assert_eq!(allowable_force_n, 744);
            }
            other => panic!("unexpected error: {:?}", other),
        }
    }

    #[test]
    fn test_pin_bearing_boundary_exact() {
        // Set up so design_force == allowable exactly → should pass
        let mut plate = valid_plate();
        plate.material = Material::Aluminum; // yield 276
        plate.pin_diameter = Millimeters(10);
        plate.plate_thickness = Millimeters(10);
        // allowable = 276 * 10 * 10 = 27,600. design = force * 2
        // So force = 13,800 → design = 27,600 = allowable → pass
        plate.expected_force_per_pin = Newtons(13800);
        assert!(validate_pin_bearing_stress(&plate).is_ok());
    }

    #[test]
    fn test_pin_bearing_boundary_just_over() {
        let mut plate = valid_plate();
        plate.material = Material::Aluminum;
        plate.pin_diameter = Millimeters(10);
        plate.plate_thickness = Millimeters(10);
        // allowable = 27,600, design = 13,801 * 2 = 27,602 → fail
        plate.expected_force_per_pin = Newtons(13801);
        assert!(validate_pin_bearing_stress(&plate).is_err());
    }

    #[test]
    fn test_pin_bearing_safety_factor_matters() {
        // A plate that would pass at 1× but fails at 2×
        let mut plate = valid_plate();
        plate.material = Material::Brass; // yield 124
        plate.pin_diameter = Millimeters(5);
        plate.plate_thickness = Millimeters(3);
        // allowable = 124 * 5 * 3 = 1,860 N
        // force = 1000 → design = 2000 > 1860 → fail
        // But at 1×: 1000 < 1860 → would pass
        plate.expected_force_per_pin = Newtons(1000);
        assert!(validate_pin_bearing_stress(&plate).is_err());
    }

    // --- Bolt bearing stress ---

    #[test]
    fn test_bolt_bearing_pass() {
        assert!(validate_bolt_bearing_stress(&valid_plate()).is_ok());
    }

    #[test]
    fn test_bolt_bearing_fail_many_pins_small_bolts() {
        let mut plate = valid_plate();
        plate.material = Material::Brass; // yield 124
        plate.bolt_size = BoltSize::M3; // 3mm nominal
        plate.plate_thickness = Millimeters(2);
        plate.pin_count = 12;
        plate.expected_force_per_pin = Newtons(500);
        // total_design = 500 * 2 * 12 = 12,000
        // per_bolt (ceiling) = ceil(12000/4) = 3,000
        // allowable = 124 * 3 * 2 = 744 → fail
        let result = validate_bolt_bearing_stress(&plate);
        assert!(result.is_err());
        match result.unwrap_err() {
            PlateValidationError::BoltBearingStressExceeded {
                force_per_bolt_n,
                allowable_per_bolt_n,
            } => {
                assert_eq!(force_per_bolt_n, 3000);
                assert_eq!(allowable_per_bolt_n, 744);
            }
            other => panic!("unexpected error: {:?}", other),
        }
    }

    // --- Plate bending stress ---

    #[test]
    fn test_bending_pass() {
        assert!(validate_plate_bending_stress(&valid_plate()).is_ok());
    }

    #[test]
    fn test_bending_fail_thin_wide_span() {
        let mut plate = valid_plate();
        plate.bolt_spacing = Millimeters(200); // wide span
        plate.plate_thickness = Millimeters(2); // very thin
        plate.bracket_width = Millimeters(250); // must be wider than spacing for edge check
        plate.pin_count = 10;
        plate.expected_force_per_pin = Newtons(2000);
        // lhs = 3 * (2000*2*10) * 200 = 3 * 40,000 * 200 = 24,000,000
        // rhs = 2 * 276 * 250 * 2 * 2 = 2 * 276 * 250 * 4 = 552,000
        // 24M > 552K → fail
        assert!(matches!(
            validate_plate_bending_stress(&plate).unwrap_err(),
            PlateValidationError::PlateBendingStressExceeded
        ));
    }

    // --- Bolt edge distance ---

    #[test]
    fn test_edge_distance_pass() {
        assert!(validate_bolt_edge_distance(&valid_plate()).is_ok());
    }

    #[test]
    fn test_edge_distance_fail_tight() {
        let mut plate = valid_plate();
        plate.bolt_spacing = Millimeters(90);
        plate.bracket_width = Millimeters(100);
        plate.bolt_size = BoltSize::M10;
        // available = 100 - 90 = 10, required = 10 * 3 = 30 → fail
        let result = validate_bolt_edge_distance(&plate);
        assert!(result.is_err());
        match result.unwrap_err() {
            PlateValidationError::BoltEdgeDistanceTooSmall {
                available_mm,
                required_mm,
            } => {
                assert_eq!(available_mm, 10);
                assert_eq!(required_mm, 30);
            }
            other => panic!("unexpected error: {:?}", other),
        }
    }

    #[test]
    fn test_edge_distance_fail_spacing_exceeds_width() {
        let mut plate = valid_plate();
        plate.bolt_spacing = Millimeters(120);
        plate.bracket_width = Millimeters(100);
        assert!(matches!(
            validate_bolt_edge_distance(&plate).unwrap_err(),
            PlateValidationError::BoltEdgeDistanceTooSmall { available_mm: 0, .. }
        ));
    }

    // --- Pin clearance ---

    #[test]
    fn test_pin_clearance_pass() {
        assert!(validate_pin_clearance(&valid_plate()).is_ok());
    }

    #[test]
    fn test_pin_clearance_fail() {
        let mut plate = valid_plate();
        plate.bracket_height = Millimeters(50);
        plate.pin_count = 6;
        plate.pin_diameter = Millimeters(10);
        // required = 6 * 10 * 3 = 180, available = 50 → fail
        let result = validate_pin_clearance(&plate);
        assert!(result.is_err());
        match result.unwrap_err() {
            PlateValidationError::InsufficientPinClearance {
                bracket_height_mm,
                required_mm,
            } => {
                assert_eq!(bracket_height_mm, 50);
                assert_eq!(required_mm, 180);
            }
            other => panic!("unexpected error: {:?}", other),
        }
    }

    // --- Minimum thickness advisory ---

    #[test]
    fn test_minimum_thickness_at_least_1() {
        let plate = valid_plate();
        assert!(minimum_thickness_mm(&plate) >= 1);
    }

    #[test]
    fn test_minimum_thickness_increases_with_force() {
        let mut plate = valid_plate();
        plate.expected_force_per_pin = Newtons(100);
        let t_low = minimum_thickness_mm(&plate);

        plate.expected_force_per_pin = Newtons(10000);
        let t_high = minimum_thickness_mm(&plate);

        assert!(t_high >= t_low);
    }

    #[test]
    fn test_minimum_thickness_brass_needs_more_than_steel() {
        let mut plate = valid_plate();
        plate.expected_force_per_pin = Newtons(5000);

        plate.material = Material::CarbonSteel;
        let t_steel = minimum_thickness_mm(&plate);

        plate.material = Material::Brass;
        let t_brass = minimum_thickness_mm(&plate);

        assert!(
            t_brass >= t_steel,
            "brass (yield 124) should need >= thickness than steel (yield 250)"
        );
    }

    // --- Material change flips pass/fail ---

    #[test]
    fn test_material_change_flips_result() {
        // A plate right at the edge for steel but failing for brass
        let mut plate = valid_plate();
        plate.pin_diameter = Millimeters(5);
        plate.plate_thickness = Millimeters(3);
        plate.expected_force_per_pin = Newtons(900);
        plate.bracket_height = Millimeters(200);
        plate.bracket_width = Millimeters(100);
        plate.bolt_spacing = Millimeters(60);
        plate.pin_count = 2;

        // Steel: allowable pin bearing = 250 * 5 * 3 = 3,750. design = 1,800. Pass.
        plate.material = Material::CarbonSteel;
        assert!(validate_pin_bearing_stress(&plate).is_ok());

        // Brass: allowable pin bearing = 124 * 5 * 3 = 1,860. design = 1,800. Pass (barely).
        plate.material = Material::Brass;
        assert!(validate_pin_bearing_stress(&plate).is_ok());

        // Increase force slightly so brass fails
        plate.expected_force_per_pin = Newtons(1000);
        // Brass: allowable = 1,860, design = 2,000 → fail
        assert!(validate_pin_bearing_stress(&plate).is_err());

        // Steel still passes: allowable = 3,750, design = 2,000
        plate.material = Material::CarbonSteel;
        assert!(validate_pin_bearing_stress(&plate).is_ok());
    }

    // --- Increasing thickness fixes failure ---

    #[test]
    fn test_increasing_thickness_fixes_bearing() {
        let mut plate = valid_plate();
        plate.material = Material::Brass;
        plate.pin_diameter = Millimeters(3);
        plate.plate_thickness = Millimeters(2);
        plate.expected_force_per_pin = Newtons(500);
        // allowable = 124 * 3 * 2 = 744, design = 1000 → fail
        assert!(validate_pin_bearing_stress(&plate).is_err());

        // Increase thickness to 3mm: allowable = 124 * 3 * 3 = 1,116 > 1,000 → pass
        plate.plate_thickness = Millimeters(3);
        assert!(validate_pin_bearing_stress(&plate).is_ok());
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
        assert_eq!(
            PlateValidationError::ExpectedForceTooSmall.to_string(),
            "expected force per pin must be greater than 0"
        );
        assert_eq!(
            PlateValidationError::ExpectedForceTooLarge.to_string(),
            "expected force per pin must not exceed 100,000 N"
        );
        assert_eq!(
            PlateValidationError::PlateBendingStressExceeded.to_string(),
            "plate bending stress exceeded: plate too thin for the applied load"
        );
    }

    // === Phase 6: Overflow and edge case tests ===

    #[test]
    fn test_force_upper_bound_rejected() {
        assert!(matches!(
            validate_expected_force(100_001).unwrap_err(),
            PlateValidationError::ExpectedForceTooLarge
        ));
    }

    #[test]
    fn test_force_at_upper_bound_accepted() {
        assert!(validate_expected_force(100_000).is_ok());
    }

    #[test]
    fn test_full_plate_rejects_excessive_force() {
        let mut plate = valid_plate();
        plate.expected_force_per_pin = Newtons(100_001);
        assert!(matches!(
            validate(&plate).unwrap_err(),
            PlateValidationError::ExpectedForceTooLarge
        ));
    }

    #[test]
    fn test_no_overflow_at_max_inputs() {
        // Max force (100kN), max pin count (12), max bolt spacing (u16::MAX)
        // Bending lhs = 3 * (100_000 * 2 * 12) * 65535 = 3 * 2_400_000 * 65535 = 471_852_000_000
        // This must not panic (fits in u64)
        let plate = ActuatorPlate {
            bolt_spacing: Millimeters(65535),
            bolt_size: BoltSize::M3,
            bracket_height: Millimeters(65535),
            bracket_width: Millimeters(65535),
            material: Material::Aluminum,
            pin_diameter: Millimeters(65535),
            pin_count: 12,
            plate_thickness: Millimeters(65535),
            expected_force_per_pin: Newtons(100_000),
        };
        // Should not panic — may pass or fail on stress, but must not overflow
        let _ = validate(&plate);
        let _ = minimum_thickness_mm(&plate);
        let _ = stress_utilization(&plate);
    }

    #[test]
    fn test_default_plate_passes_all_checks() {
        let plate = ActuatorPlate::default();
        assert!(validate(&plate).is_ok());
    }

    #[test]
    fn test_extreme_force_fails_default_plate() {
        let plate = ActuatorPlate { expected_force_per_pin: Newtons(100_000), ..Default::default() };
        assert!(validate(&plate).is_err());
    }

    // Test matrix from plan section 6.2
    #[test]
    fn test_pin_bearing_matrix_aluminum_10_8_500() {
        let mut plate = valid_plate();
        plate.material = Material::Aluminum;
        plate.pin_diameter = Millimeters(10);
        plate.plate_thickness = Millimeters(8);
        plate.expected_force_per_pin = Newtons(500);
        // allowable = 276 * 10 * 8 = 22,080, design = 1000 → PASS
        assert!(validate_pin_bearing_stress(&plate).is_ok());
    }

    #[test]
    fn test_pin_bearing_matrix_brass_5_3_300() {
        let mut plate = valid_plate();
        plate.material = Material::Brass;
        plate.pin_diameter = Millimeters(5);
        plate.plate_thickness = Millimeters(3);
        plate.expected_force_per_pin = Newtons(300);
        // allowable = 124 * 5 * 3 = 1,860, design = 600 → PASS
        assert!(validate_pin_bearing_stress(&plate).is_ok());
    }

    #[test]
    fn test_pin_bearing_matrix_brass_3_2_200() {
        let mut plate = valid_plate();
        plate.material = Material::Brass;
        plate.pin_diameter = Millimeters(3);
        plate.plate_thickness = Millimeters(2);
        plate.expected_force_per_pin = Newtons(200);
        // allowable = 124 * 3 * 2 = 744, design = 400 → PASS
        assert!(validate_pin_bearing_stress(&plate).is_ok());
    }

    #[test]
    fn test_pin_bearing_matrix_brass_3_2_500() {
        let mut plate = valid_plate();
        plate.material = Material::Brass;
        plate.pin_diameter = Millimeters(3);
        plate.plate_thickness = Millimeters(2);
        plate.expected_force_per_pin = Newtons(500);
        // allowable = 124 * 3 * 2 = 744, design = 1000 → FAIL
        assert!(validate_pin_bearing_stress(&plate).is_err());
    }

    // Stress utilization tests
    #[test]
    fn test_utilization_ratios_are_positive() {
        let plate = valid_plate();
        let u = stress_utilization(&plate);
        assert!(u.pin_bearing > 0.0);
        assert!(u.bolt_bearing > 0.0);
        assert!(u.bending >= 0.0);
    }

    #[test]
    fn test_utilization_below_one_for_valid_plate() {
        let plate = valid_plate();
        let u = stress_utilization(&plate);
        assert!(u.pin_bearing < 1.0, "pin_bearing={}", u.pin_bearing);
        assert!(u.bolt_bearing < 1.0, "bolt_bearing={}", u.bolt_bearing);
        assert!(u.bending < 1.0, "bending={}", u.bending);
    }

    #[test]
    fn test_utilization_increases_with_force() {
        let mut plate = valid_plate();
        plate.expected_force_per_pin = Newtons(100);
        let u_low = stress_utilization(&plate);

        plate.expected_force_per_pin = Newtons(10000);
        let u_high = stress_utilization(&plate);

        assert!(u_high.pin_bearing > u_low.pin_bearing);
        assert!(u_high.bolt_bearing > u_low.bolt_bearing);
        assert!(u_high.bending > u_low.bending);
    }
}
