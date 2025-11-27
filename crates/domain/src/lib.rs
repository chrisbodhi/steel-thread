#![no_std]

use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, Deserialize, Serialize, PartialEq, PartialOrd)]
pub struct Millimeters(pub u16);

#[derive(Copy, Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct ActuatorPlate {
    pub bolt_spacing: Millimeters,
    pub bolt_diameter: Millimeters,
    pub bracket_height: Millimeters,
    pub pin_diameter: Millimeters,
    pub plate_thickness: Millimeters,
}

impl ActuatorPlate {
    pub fn new(
        bolt_spacing: Millimeters,
        bolt_diameter: Millimeters,
        bracket_height: Millimeters,
        pin_diameter: Millimeters,
        plate_thickness: Millimeters,
    ) -> Self {
        ActuatorPlate {
            bolt_spacing,
            bolt_diameter,
            bracket_height,
            pin_diameter,
            plate_thickness,
        }
    }

    pub fn default() -> Self {
        ActuatorPlate {
            bolt_spacing: Millimeters(60),
            bolt_diameter: Millimeters(10),
            bracket_height: Millimeters(40),
            pin_diameter: Millimeters(10),
            plate_thickness: Millimeters(8),
        }
    }
}
