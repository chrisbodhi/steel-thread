#![cfg_attr(not(feature = "openapi"), no_std)]

use serde::{Deserialize, Serialize};

/// A type-safe wrapper for millimeter measurements.
#[derive(Copy, Clone, Debug, Deserialize, Serialize, PartialEq, PartialOrd)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "openapi", schema(example = 60))]
pub struct Millimeters(pub u16);

/// Standard ISO metric bolt sizes with clearance hole diameters.
///
/// Each variant represents a standard metric bolt size (e.g., M3 = 3mm nominal diameter).
/// The clearance hole diameter is sized to allow the bolt to pass through freely.
#[derive(Copy, Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "UPPERCASE")]
pub enum BoltSize {
    /// M3 bolt (3mm nominal, 3.4mm clearance hole)
    M3,
    /// M4 bolt (4mm nominal, 4.5mm clearance hole)
    M4,
    /// M5 bolt (5mm nominal, 5.5mm clearance hole)
    M5,
    /// M6 bolt (6mm nominal, 6.6mm clearance hole)
    M6,
    /// M8 bolt (8mm nominal, 9.0mm clearance hole)
    M8,
    /// M10 bolt (10mm nominal, 11.0mm clearance hole)
    M10,
    /// M12 bolt (12mm nominal, 13.5mm clearance hole)
    M12,
}

impl BoltSize {
    /// Returns the nominal diameter of the bolt in millimeters.
    pub const fn nominal_diameter_mm(self) -> u16 {
        match self {
            BoltSize::M3 => 3,
            BoltSize::M4 => 4,
            BoltSize::M5 => 5,
            BoltSize::M6 => 6,
            BoltSize::M8 => 8,
            BoltSize::M10 => 10,
            BoltSize::M12 => 12,
        }
    }

    /// Returns the clearance hole diameter in millimeters.
    ///
    /// Clearance holes are sized to allow the bolt to pass through freely
    /// without binding. These are standard engineering clearances for close fit.
    pub const fn clearance_hole_diameter_mm(self) -> f32 {
        match self {
            BoltSize::M3 => 3.4,
            BoltSize::M4 => 4.5,
            BoltSize::M5 => 5.5,
            BoltSize::M6 => 6.6,
            BoltSize::M8 => 9.0,
            BoltSize::M10 => 11.0,
            BoltSize::M12 => 13.5,
        }
    }
}

/// Materials suitable for actuator mounting plates.
///
/// Each material has different properties affecting strength, weight,
/// corrosion resistance, and cost. The choice of material affects
/// manufacturing tolerances and appearance.
#[derive(Copy, Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub enum Material {
    /// Aluminum 6061-T6: Lightweight, excellent corrosion resistance,
    /// good machinability. Most common choice for general-purpose plates.
    Aluminum,
    /// Stainless Steel 304: Higher strength than aluminum, excellent
    /// corrosion resistance. Heavier but more durable.
    StainlessSteel,
    /// Carbon Steel: Highest strength, cost-effective. Requires coating
    /// or treatment for corrosion protection.
    CarbonSteel,
    /// Brass: Good corrosion resistance, decorative appearance.
    /// Suitable for specific environments or aesthetic requirements.
    Brass,
}

impl Material {
    /// Returns the material name as a lowercase string for KCL.
    pub const fn as_kcl_str(&self) -> &'static str {
        match self {
            Material::Aluminum => "aluminum",
            Material::StainlessSteel => "stainless_steel",
            Material::CarbonSteel => "carbon_steel",
            Material::Brass => "brass",
        }
    }
}

/// Configuration for an actuator plate assembly.
///
/// Defines the physical dimensions and parameters for manufacturing
/// a custom actuator plate with mounting bolts and actuator pins.
#[derive(Copy, Clone, Debug, Deserialize, Serialize, PartialEq)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ActuatorPlate {
    /// Distance between mounting bolt centers (in millimeters).
    ///
    /// Used for the bolt hole pattern layout. Determines the spacing
    /// between mounting points on the plate.
    pub bolt_spacing: Millimeters,

    /// Standard ISO metric bolt size for mounting holes.
    ///
    /// Determines the size of clearance holes used to secure the plate
    /// to the mounting surface. Uses standard metric bolt sizes (M3-M12).
    pub bolt_size: BoltSize,

    /// Height of the mounting bracket (in millimeters).
    ///
    /// Vertical dimension of the bracket that holds the actuator.
    pub bracket_height: Millimeters,

    /// Width of the mounting bracket (in millimeters).
    ///
    /// Horizontal dimension of the bracket that holds the actuator.
    pub bracket_width: Millimeters,

    /// Material for the plate.
    ///
    /// Affects strength, weight, corrosion resistance, and appearance.
    /// Common choices are aluminum (lightweight), stainless steel (durable),
    /// carbon steel (strong), or brass (decorative).
    pub material: Material,

    /// Diameter of actuator pivot pins (in millimeters).
    ///
    /// Separate from mounting bolts. These pins are used for the actuator
    /// mechanism's pivot points and articulation.
    pub pin_diameter: Millimeters,

    /// Number of actuator pins.
    ///
    /// Count of pivot pins required for the actuator mechanism.
    /// Must be between 1 and 12 inclusive.
    #[cfg_attr(feature = "openapi", schema(example = 6))]
    pub pin_count: u16,

    /// Thickness of the base plate material (in millimeters).
    ///
    /// Determines the structural rigidity and extrusion depth of the plate.
    pub plate_thickness: Millimeters,
}

impl ActuatorPlate {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        bolt_spacing: Millimeters,
        bolt_size: BoltSize,
        bracket_height: Millimeters,
        bracket_width: Millimeters,
        material: Material,
        pin_diameter: Millimeters,
        pin_count: u16,
        plate_thickness: Millimeters,
    ) -> Self {
        ActuatorPlate {
            bolt_spacing,
            bolt_size,
            bracket_height,
            bracket_width,
            material,
            pin_diameter,
            pin_count,
            plate_thickness,
        }
    }

    /// Generate a deterministic cache key based on plate configuration.
    /// Returns a string in the format "plate-{16_hex_chars}" derived from SHA-256 hash.
    #[cfg(feature = "openapi")]
    pub fn cache_key(&self) -> String {
        use sha2::{Digest, Sha256};

        let mut hasher = Sha256::new();
        hasher.update(self.bolt_spacing.0.to_le_bytes());
        hasher.update(self.bolt_size.nominal_diameter_mm().to_le_bytes());
        hasher.update(self.bracket_height.0.to_le_bytes());
        hasher.update(self.bracket_width.0.to_le_bytes());
        hasher.update(self.material.as_kcl_str().as_bytes());
        hasher.update(self.pin_diameter.0.to_le_bytes());
        hasher.update(self.pin_count.to_le_bytes());
        hasher.update(self.plate_thickness.0.to_le_bytes());

        let result = hasher.finalize();
        format!("plate-{}", hex::encode(&result[..8]))
    }
}

impl Default for ActuatorPlate {
    fn default() -> Self {
        ActuatorPlate {
            bolt_spacing: Millimeters(60),
            bolt_size: BoltSize::M10,
            bracket_height: Millimeters(400),
            bracket_width: Millimeters(300),
            material: Material::Aluminum,
            pin_diameter: Millimeters(10),
            pin_count: 6,
            plate_thickness: Millimeters(8),
        }
    }
}

#[cfg(all(test, feature = "openapi"))]
mod tests {
    use super::*;

    #[test]
    fn test_cache_key_is_deterministic() {
        let plate = ActuatorPlate::default();
        let key1 = plate.cache_key();
        let key2 = plate.cache_key();
        assert_eq!(key1, key2);
    }

    #[test]
    fn test_cache_key_format() {
        let plate = ActuatorPlate::default();
        let key = plate.cache_key();
        assert!(key.starts_with("plate-"));
        assert_eq!(key.len(), 6 + 16); // "plate-" + 16 hex chars
    }

    #[test]
    fn test_cache_key_differs_for_different_plates() {
        let plate1 = ActuatorPlate::default();
        let mut plate2 = ActuatorPlate::default();
        plate2.bolt_spacing = Millimeters(61);

        assert_ne!(plate1.cache_key(), plate2.cache_key());
    }

    #[test]
    fn test_cache_key_differs_for_different_materials() {
        let plate1 = ActuatorPlate::default();
        let mut plate2 = ActuatorPlate::default();
        plate2.material = Material::StainlessSteel;

        assert_ne!(plate1.cache_key(), plate2.cache_key());
    }

    #[test]
    fn test_cache_key_same_for_equal_plates() {
        let plate1 = ActuatorPlate::default();
        let plate2 = ActuatorPlate::default();
        assert_eq!(plate1.cache_key(), plate2.cache_key());
    }

    #[test]
    fn test_material_kcl_str() {
        assert_eq!(Material::Aluminum.as_kcl_str(), "aluminum");
        assert_eq!(Material::StainlessSteel.as_kcl_str(), "stainless_steel");
        assert_eq!(Material::CarbonSteel.as_kcl_str(), "carbon_steel");
        assert_eq!(Material::Brass.as_kcl_str(), "brass");
    }
}
