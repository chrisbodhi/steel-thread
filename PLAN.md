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

### In Progress
- [ ] Wire up form controls (sliders) for plate parameters:
  - [ ] `bolt_spacing` in mm (default: 60, min: 10, max: 200)
  - [ ] `bolt_diameter` in mm (default: 10, min: 3, max: 20)
  - [ ] `bracket_height` in mm (default: 40, min: 10, max: 100)
  - [ ] `bracket_width` in mm (default: 30, min: 10, max: 100)
  - [ ] `pin_diameter` in mm (default: 10, min: 5, max: 30)
  - [ ] `plate_thickness` in mm (default: 8, min: 2, max: 20)
- [ ] Connect form to `/api/generate` endpoint
- [ ] Display validation errors from API
- [ ] Real-time validation feedback (consider WASM client-side validation)

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
