//! WebAssembly bindings for validation functions.
//!
//! This module provides JavaScript-friendly validation functions that can be
//! called from the frontend for real-time field validation.

extern crate alloc;

use alloc::string::{String, ToString};
use wasm_bindgen::prelude::*;

use crate::{
    validate_bolt_size, validate_bolt_spacing, validate_bracket_height, validate_bracket_width,
    validate_material, validate_pin_count, validate_pin_diameter, validate_plate_thickness,
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
