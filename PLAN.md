# Plan

## Current Status: React Frontend + Validation API

This branch (`remove-ssr`) replaced the server-side rendered UI with a React SPA.

### Completed
- [x] Axum API server serving on http://localhost:3030
- [x] REST API with model generation endpoint (`POST /api/generate`)
- [x] React SPA with Bun (development server with HMR)
- [x] Production build: Rust serves API + static frontend from single process
- [x] Shared validation crate (`no_std` compatible)
- [x] Basic form UI foundation (not yet wired to sliders)

### Completed Form Integration
- [x] Wire up form controls for plate parameters:
  - [x] `bolt_spacing` in mm (number input with validation)
  - [x] `bolt_size` - Standard ISO metric sizes (M3, M4, M5, M6, M8, M10, M12) via dropdown
  - [x] `bracket_height` in mm (number input with validation)
  - [x] `bracket_width` in mm (number input with validation)
  - [x] `pin_diameter` in mm (number input with validation)
  - [x] `pin_count` - Count between 1-12 (number input with validation)
  - [x] `plate_thickness` in mm (number input with validation)
- [x] Connect form to `/api/generate` endpoint
- [x] Display validation errors from API
- [x] Real-time WASM validation feedback (instant client-side validation)

### Next Steps (Future Work)
- [ ] Generate visual preview of plate (2D or 3D)
- [ ] User adjusts parameters and sees updated preview
- [ ] User selects quantity
- [ ] Generate quote based on geometry and quantity
- [ ] Order flow with payment processing
- [ ] CAD file generation (STEP/STL) for download
- [ ] Integration with manufacturing services (SendCutSend/Xometry)

## Architecture Notes

**Current**: Web form → REST API → Validation → Response (success/errors)

**Next Phase**: Add CAD generation
- User submits form
- Server validates parameters
- Server generates CAD model (OnShape/FreeCAD/CadQuery/truck)
- Returns 3D preview + downloadable STEP file
- Target: <10 seconds end-to-end

See [LEARNING.md](./LEARNING.md) for full research questions and success criteria.
