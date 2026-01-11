#![no_std]

use serde::{Deserialize, Serialize};

/// A type-safe wrapper for millimeter measurements.
#[derive(Copy, Clone, Debug, Deserialize, Serialize, PartialEq, PartialOrd)]
pub struct Millimeters(pub u16);

/// Configuration for an actuator plate assembly.
///
/// Defines the physical dimensions and parameters for manufacturing
/// a custom actuator plate with mounting bolts and actuator pins.
#[derive(Copy, Clone, Debug, Deserialize, Serialize, PartialEq)]
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

    pub fn default() -> Self {
        ActuatorPlate {
            bolt_spacing: Millimeters(60),
            bolt_diameter: Millimeters(10),
            bracket_height: Millimeters(40),
            bracket_width: Millimeters(30),
            pin_diameter: Millimeters(10),
            pin_count: 6,
            plate_thickness: Millimeters(8),
        }
    }
}
