/**
 * Validation module using WebAssembly for fast, client-side validation.
 *
 * This module wraps the WASM validation functions and provides a TypeScript-friendly API.
 */

import init, {
  wasm_validate_bolt_spacing,
  wasm_validate_bolt_diameter,
  wasm_validate_bracket_height,
  wasm_validate_bracket_width,
  wasm_validate_pin_diameter,
  wasm_validate_pin_count,
  wasm_validate_plate_thickness,
} from '../wasm-validation/validation.js';

// Initialize WASM module on first import
let wasmInitialized = false;
let wasmInitPromise: Promise<void> | null = null;

/**
 * Initialize the WASM validation module.
 * This is called automatically on first validation, but can be called manually for better performance.
 */
export async function initValidation(): Promise<void> {
  if (wasmInitialized) {
    return;
  }

  if (wasmInitPromise) {
    return wasmInitPromise;
  }

  wasmInitPromise = (async () => {
    try {
      // Try to load WASM from the expected locations
      // In development: /wasm-validation/validation_bg.wasm
      // In production: /wasm-validation/validation_bg.wasm
      const wasmUrl = '/wasm-validation/validation_bg.wasm';
      await init(wasmUrl);
      wasmInitialized = true;
    } catch (error) {
      console.error('Failed to initialize WASM validation module:', error);
      throw error;
    }
  })();

  return wasmInitPromise;
}

/**
 * Result of a validation operation.
 */
export type ValidationResult = {
  valid: boolean;
  error?: string;
};

/**
 * Helper function to wrap WASM validation calls.
 */
function validate(fn: () => void): ValidationResult {
  try {
    fn();
    return { valid: true };
  } catch (error) {
    return {
      valid: false,
      error: error instanceof Error ? error.message : String(error),
    };
  }
}

/**
 * Validate bolt spacing value.
 */
export async function validateBoltSpacing(value: number): Promise<ValidationResult> {
  await initValidation();
  return validate(() => wasm_validate_bolt_spacing(value));
}

/**
 * Validate bolt diameter value.
 */
export async function validateBoltDiameter(value: number): Promise<ValidationResult> {
  await initValidation();
  return validate(() => wasm_validate_bolt_diameter(value));
}

/**
 * Validate bracket height value.
 */
export async function validateBracketHeight(value: number): Promise<ValidationResult> {
  await initValidation();
  return validate(() => wasm_validate_bracket_height(value));
}

/**
 * Validate bracket width value.
 */
export async function validateBracketWidth(value: number): Promise<ValidationResult> {
  await initValidation();
  return validate(() => wasm_validate_bracket_width(value));
}

/**
 * Validate pin diameter value.
 */
export async function validatePinDiameter(value: number): Promise<ValidationResult> {
  await initValidation();
  return validate(() => wasm_validate_pin_diameter(value));
}

/**
 * Validate pin count value.
 */
export async function validatePinCount(value: number): Promise<ValidationResult> {
  await initValidation();
  return validate(() => wasm_validate_pin_count(value));
}

/**
 * Validate plate thickness value.
 */
export async function validatePlateThickness(value: number): Promise<ValidationResult> {
  await initValidation();
  return validate(() => wasm_validate_plate_thickness(value));
}
