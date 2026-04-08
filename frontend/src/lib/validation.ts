/**
 * Validation module using WebAssembly for fast, client-side validation.
 *
 * This module wraps the WASM validation functions and provides a TypeScript-friendly API.
 */

import init, {
  wasm_validate_bolt_spacing,
  wasm_validate_bolt_size,
  wasm_validate_bracket_height,
  wasm_validate_bracket_width,
  wasm_validate_material,
  wasm_validate_pin_diameter,
  wasm_validate_pin_count,
  wasm_validate_plate_thickness,
  wasm_validate_expected_force,
  wasm_validate_stress,
  wasm_minimum_thickness,
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
 * Validate bolt size value.
 * Accepts standard ISO metric bolt sizes: M3, M4, M5, M6, M8, M10, M12.
 */
export async function validateBoltSize(value: string): Promise<ValidationResult> {
  await initValidation();
  return validate(() => wasm_validate_bolt_size(value));
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

/**
 * Validate material value.
 * Accepts: aluminum, stainless_steel, carbon_steel, brass.
 */
export async function validateMaterial(value: string): Promise<ValidationResult> {
  await initValidation();
  return validate(() => wasm_validate_material(value));
}

/**
 * Validate expected force per pin value (in Newtons).
 */
export async function validateExpectedForce(value: number): Promise<ValidationResult> {
  await initValidation();
  return validate(() => wasm_validate_expected_force(value));
}

/**
 * Run full stress analysis on a plate configuration.
 *
 * Checks all basic constraints plus engineering stress checks
 * (pin bearing, bolt bearing, plate bending, edge distance, pin clearance)
 * with a 2x safety factor on forces.
 */
export async function validateStress(params: {
  boltSpacing: number;
  boltSize: string;
  bracketHeight: number;
  bracketWidth: number;
  material: string;
  pinDiameter: number;
  pinCount: number;
  plateThickness: number;
  expectedForcePerPin: number;
}): Promise<ValidationResult> {
  await initValidation();
  return validate(() =>
    wasm_validate_stress(
      params.boltSpacing,
      params.boltSize,
      params.bracketHeight,
      params.bracketWidth,
      params.material,
      params.pinDiameter,
      params.pinCount,
      params.plateThickness,
      params.expectedForcePerPin,
    ),
  );
}

/**
 * Get the minimum recommended plate thickness (mm) for the given configuration.
 *
 * Returns the smallest thickness that satisfies bearing and bending constraints.
 * Returns 0 if the WASM module fails to compute.
 */
export async function getMinimumThickness(params: {
  boltSpacing: number;
  boltSize: string;
  bracketWidth: number;
  material: string;
  pinDiameter: number;
  pinCount: number;
  expectedForcePerPin: number;
}): Promise<number> {
  await initValidation();
  try {
    return wasm_minimum_thickness(
      params.boltSpacing,
      params.boltSize,
      params.bracketWidth,
      params.material,
      params.pinDiameter,
      params.pinCount,
      params.expectedForcePerPin,
    );
  } catch {
    return 0;
  }
}
