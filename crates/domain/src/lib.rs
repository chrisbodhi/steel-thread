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
    pub fn new(
        bolt_spacing: Millimeters,
        bolt_size: BoltSize,
        bracket_height: Millimeters,
        bracket_width: Millimeters,
        pin_diameter: Millimeters,
        pin_count: u16,
        plate_thickness: Millimeters,
    ) -> Self {
        ActuatorPlate {
            bolt_spacing,
            bolt_size,
            bracket_height,
            bracket_width,
            pin_diameter,
            pin_count,
            plate_thickness,
        }
    }
}

impl Default for ActuatorPlate {
    fn default() -> Self {
        ActuatorPlate {
            bolt_spacing: Millimeters(60),
            bolt_size: BoltSize::M10,
            bracket_height: Millimeters(400),
            bracket_width: Millimeters(300),
            pin_diameter: Millimeters(10),
            pin_count: 6,
            plate_thickness: Millimeters(8),
        }
    }
}
