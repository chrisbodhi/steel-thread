//! WebAssembly bindings for validation functions.
//!
//! This module provides JavaScript-friendly validation functions that can be
//! called from the frontend for real-time field validation.

extern crate alloc;

use alloc::string::{String, ToString};
use wasm_bindgen::prelude::*;

use crate::{
    validate_bolt_size, validate_bolt_spacing, validate_bracket_height, validate_bracket_width,
    validate_expected_force, validate_material, validate_pin_count, validate_pin_diameter,
    validate_plate_thickness,
};

/// Validate bolt spacing value.
///
/// Returns Ok(()) if valid, or an error message if invalid.
#[wasm_bindgen]
pub fn wasm_validate_bolt_spacing(value: u16) -> Result<(), String> {
    validate_bolt_spacing(value).map_err(|e| e.to_string())
}

/// Validate bolt size value.
///
/// Accepts standard ISO metric bolt sizes: M3, M4, M5, M6, M8, M10, M12.
/// Returns Ok(()) if valid, or an error message if invalid.
#[wasm_bindgen]
pub fn wasm_validate_bolt_size(value: &str) -> Result<(), String> {
    validate_bolt_size(value).map_err(|e| e.to_string())
}

/// Validate bracket height value.
///
/// Returns Ok(()) if valid, or an error message if invalid.
#[wasm_bindgen]
pub fn wasm_validate_bracket_height(value: u16) -> Result<(), String> {
    validate_bracket_height(value).map_err(|e| e.to_string())
}

/// Validate bracket width value.
///
/// Returns Ok(()) if valid, or an error message if invalid.
#[wasm_bindgen]
pub fn wasm_validate_bracket_width(value: u16) -> Result<(), String> {
    validate_bracket_width(value).map_err(|e| e.to_string())
}

/// Validate pin diameter value.
///
/// Returns Ok(()) if valid, or an error message if invalid.
#[wasm_bindgen]
pub fn wasm_validate_pin_diameter(value: u16) -> Result<(), String> {
    validate_pin_diameter(value).map_err(|e| e.to_string())
}

/// Validate pin count value.
///
/// Returns Ok(()) if valid, or an error message if invalid.
#[wasm_bindgen]
pub fn wasm_validate_pin_count(value: u16) -> Result<(), String> {
    validate_pin_count(value).map_err(|e| e.to_string())
}

/// Validate plate thickness value.
///
/// Returns Ok(()) if valid, or an error message if invalid.
#[wasm_bindgen]
pub fn wasm_validate_plate_thickness(value: u16) -> Result<(), String> {
    validate_plate_thickness(value).map_err(|e| e.to_string())
}

/// Validate material value.
///
/// Accepts: aluminum, stainless_steel, carbon_steel, brass.
/// Returns Ok(()) if valid, or an error message if invalid.
#[wasm_bindgen]
pub fn wasm_validate_material(value: &str) -> Result<(), String> {
    validate_material(value).map_err(|e| e.to_string())
}

/// Validate expected force per pin value.
///
/// Returns Ok(()) if valid, or an error message if invalid.
#[wasm_bindgen]
pub fn wasm_validate_expected_force(value: u32) -> Result<(), String> {
    validate_expected_force(value).map_err(|e| e.to_string())
}

/// Run full stress analysis on a plate configuration.
///
/// Takes all plate parameters as flat values (wasm-bindgen doesn't support structs).
/// Runs both basic constraint checks and engineering stress checks with 2x safety factor.
/// Returns Ok(()) if the plate passes all checks, or Err(String) with the first violation.
#[wasm_bindgen]
pub fn wasm_validate_stress(
    bolt_spacing: u16,
    bolt_size: &str,
    bracket_height: u16,
    bracket_width: u16,
    material: &str,
    pin_diameter: u16,
    pin_count: u16,
    plate_thickness: u16,
    expected_force_per_pin: u32,
) -> Result<(), String> {
    use domain::{ActuatorPlate, Millimeters, Newtons};

    let bolt_size = parse_bolt_size(bolt_size)?;
    let material = parse_material(material)?;

    let plate = ActuatorPlate {
        bolt_spacing: Millimeters(bolt_spacing),
        bolt_size,
        bracket_height: Millimeters(bracket_height),
        bracket_width: Millimeters(bracket_width),
        material,
        pin_diameter: Millimeters(pin_diameter),
        pin_count,
        plate_thickness: Millimeters(plate_thickness),
        expected_force_per_pin: Newtons(expected_force_per_pin),
    };

    crate::validate(&plate).map_err(|e| e.to_string())
}

/// Returns the minimum recommended plate thickness in mm.
///
/// Given material, geometry, and force, computes the smallest thickness that
/// would satisfy all bearing and bending stress checks.
#[wasm_bindgen]
pub fn wasm_minimum_thickness(
    bolt_spacing: u16,
    bolt_size: &str,
    bracket_width: u16,
    material: &str,
    pin_diameter: u16,
    pin_count: u16,
    expected_force_per_pin: u32,
) -> Result<u16, String> {
    use domain::{ActuatorPlate, Millimeters, Newtons};

    let bolt_size = parse_bolt_size(bolt_size)?;
    let material = parse_material(material)?;

    // Use placeholder values for fields not needed by minimum_thickness_mm
    let plate = ActuatorPlate {
        bolt_spacing: Millimeters(bolt_spacing),
        bolt_size,
        bracket_height: Millimeters(1000), // not used in thickness calc
        bracket_width: Millimeters(bracket_width),
        material,
        pin_diameter: Millimeters(pin_diameter),
        pin_count,
        plate_thickness: Millimeters(1), // not used — we're computing this
        expected_force_per_pin: Newtons(expected_force_per_pin),
    };

    Ok(crate::minimum_thickness_mm(&plate))
}

/// Parse a bolt size string into a BoltSize enum.
fn parse_bolt_size(value: &str) -> Result<domain::BoltSize, String> {
    let trimmed = value.trim();
    match trimmed {
        "M3" | "m3" => Ok(domain::BoltSize::M3),
        "M4" | "m4" => Ok(domain::BoltSize::M4),
        "M5" | "m5" => Ok(domain::BoltSize::M5),
        "M6" | "m6" => Ok(domain::BoltSize::M6),
        "M8" | "m8" => Ok(domain::BoltSize::M8),
        "M10" | "m10" => Ok(domain::BoltSize::M10),
        "M12" | "m12" => Ok(domain::BoltSize::M12),
        _ => Err(crate::PlateValidationError::BoltSizeInvalid.to_string()),
    }
}

/// Parse a material string into a Material enum.
fn parse_material(value: &str) -> Result<domain::Material, String> {
    let trimmed = value.trim();
    match trimmed {
        "aluminum" | "Aluminum" | "ALUMINUM" => Ok(domain::Material::Aluminum),
        "stainless_steel" | "Stainless_Steel" | "STAINLESS_STEEL" | "stainlessSteel"
        | "StainlessSteel" => Ok(domain::Material::StainlessSteel),
        "carbon_steel" | "Carbon_Steel" | "CARBON_STEEL" | "carbonSteel" | "CarbonSteel" => {
            Ok(domain::Material::CarbonSteel)
        }
        "brass" | "Brass" | "BRASS" => Ok(domain::Material::Brass),
        _ => Err(crate::PlateValidationError::MaterialInvalid.to_string()),
    }
}
