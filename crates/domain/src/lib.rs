#![cfg_attr(not(feature = "openapi"), no_std)]

use serde::{Deserialize, Serialize};

/// A type-safe wrapper for millimeter measurements.
#[derive(Copy, Clone, Debug, Deserialize, Serialize, PartialEq, PartialOrd)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "openapi", schema(example = 60))]
pub struct Millimeters(pub u16);

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

    /// Diameter of mounting bolts (in millimeters).
    ///
    /// Determines the size of mounting holes used to secure the plate
    /// to the mounting surface.
    pub bolt_diameter: Millimeters,

    /// Height of the mounting bracket (in millimeters).
    ///
    /// Vertical dimension of the bracket that holds the actuator.
    pub bracket_height: Millimeters,

    /// Width of the mounting bracket (in millimeters).
    ///
    /// Horizontal dimension of the bracket that holds the actuator.
    pub bracket_width: Millimeters,

    /// Diameter of actuator pivot pins (in millimeters).
    ///
    /// Separate from mounting bolts. These pins are used for the actuator
    /// mechanism's pivot points and articulation.
    pub pin_diameter: Millimeters,

    /// Number of actuator pins.
    ///
    /// Count of pivot pins required for the actuator mechanism.
    /// Must be between 1 and 12 inclusive.
    pub pin_count: u16,

    /// Thickness of the base plate material (in millimeters).
    ///
    /// Determines the structural rigidity and extrusion depth of the plate.
    pub plate_thickness: Millimeters,
}

impl ActuatorPlate {
    pub fn new(
        bolt_spacing: Millimeters,
        bolt_diameter: Millimeters,
        bracket_height: Millimeters,
        bracket_width: Millimeters,
        pin_diameter: Millimeters,
        pin_count: u16,
        plate_thickness: Millimeters,
    ) -> Self {
        ActuatorPlate {
            bolt_spacing,
            bolt_diameter,
            bracket_height,
            bracket_width,
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
        hasher.update(self.bolt_diameter.0.to_le_bytes());
        hasher.update(self.bracket_height.0.to_le_bytes());
        hasher.update(self.bracket_width.0.to_le_bytes());
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
            bolt_diameter: Millimeters(10),
            bracket_height: Millimeters(400),
            bracket_width: Millimeters(300),
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
    fn test_cache_key_same_for_equal_plates() {
        let plate1 = ActuatorPlate::default();
        let plate2 = ActuatorPlate::default();
        assert_eq!(plate1.cache_key(), plate2.cache_key());
    }
}
