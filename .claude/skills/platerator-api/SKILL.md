---
name: platerator-api
description: Call the Platerator REST API to validate actuator plate parameters, generate STEP and glTF model files, and download the results. Use when the user asks about integrating with Platerator, validating a plate config, or generating a mounting plate programmatically.
---

# Platerator API

Platerator generates custom mounting plates for linear actuators. The API takes
a plate configuration, validates it against stress constraints, then produces
downloadable STEP (manufacturing) and glTF (3D preview) files.

Base URL in local development: `http://localhost:3030`.
Interactive docs: `GET /api/docs`. Machine-readable spec: `GET /api/openapi.json`.

## When to use this skill

- The user wants to call Platerator from their own code, Postman, `curl`, etc.
- The user asks "what fields does the API take" or "how do I validate a plate".
- The user is writing client code that submits a plate config and downloads files.

## Endpoints

| Method | Path                                   | Purpose                                  |
| ------ | -------------------------------------- | ---------------------------------------- |
| GET    | `/api/health`                          | Liveness check                           |
| POST   | `/api/validate`                        | Validate a plate config without generating |
| POST   | `/api/generate`                        | Generate STEP and glTF files             |
| GET    | `/api/download/step/{session_id}`      | Download the generated STEP file         |
| GET    | `/api/download/gltf/{session_id}`      | Download the generated glTF file         |
| GET    | `/api/docs`                            | Swagger UI                               |
| GET    | `/api/openapi.json`                    | OpenAPI 3.0 spec                         |

## Request body: `ActuatorPlate`

Both `/api/validate` and `/api/generate` take the same JSON body:

```json
{
  "bolt_spacing": 60,
  "bolt_size": "M10",
  "bracket_height": 40,
  "bracket_width": 30,
  "material": "aluminum",
  "pin_diameter": 10,
  "pin_count": 6,
  "plate_thickness": 8,
  "expected_force_per_pin": 500
}
```

Field reference:

| Field             | Type                 | Unit   | Notes                                                                                      |
| ----------------- | -------------------- | ------ | ------------------------------------------------------------------------------------------ |
| `bolt_spacing`    | integer (u16)        | mm     | Distance between mounting bolt centers.                                                    |
| `bolt_size`       | enum                 | —      | Standard ISO metric: `M3`, `M4`, `M5`, `M6`, `M8`, `M10`, `M12`. Serialized uppercase.     |
| `bracket_height`  | integer (u16)        | mm     | Vertical bracket dimension.                                                                |
| `bracket_width`   | integer (u16)        | mm     | Horizontal bracket dimension.                                                              |
| `material`        | enum                 | —      | `aluminum`, `stainless_steel`, `carbon_steel`, `brass` (snake_case).                       |
| `pin_diameter`    | integer (u16)        | mm     | Actuator pivot pin diameter.                                                               |
| `pin_count`       | integer (u16)        | count  | Number of pins, 1–12.                                                                      |
| `plate_thickness` | integer (u16)        | mm     | Plate thickness.                                                                           |
| `expected_force_per_pin` | integer (u32) | N      | Nominal force per pin. Stress checks apply a 2× safety factor internally.                  |

## Responses

### `POST /api/validate`

**200 OK** — `ValidationSuccessResponse`:
```json
{
  "valid": true,
  "message": "Actuator plate parameters are valid",
  "stress_summary": {
    "safety_factor": 2.0,
    "pin_bearing_utilization": 0.42,
    "bolt_bearing_utilization": 0.31,
    "bending_utilization": 0.58,
    "minimum_thickness_mm": 6
  }
}
```

**400 Bad Request** — `ValidationErrorResponse`:
```json
{
  "valid": false,
  "errors": [
    { "message": "plate bending stress exceeded", "fields": ["plateThickness", "expectedForcePerPin"] }
  ],
  "minimum_thickness_mm": 12
}
```

`minimum_thickness_mm` is only populated when a stress constraint is the
failing reason; use it to suggest a self-healing retry.

### `POST /api/generate`

**200 OK** — `GenerateSuccessResponse` (plus `X-Cache: HIT|MISS` header):
```json
{
  "success": true,
  "message": "Model files generated successfully",
  "download_url": "/api/download/step/<session_id>",
  "gltf_url": "/api/download/gltf/<session_id>",
  "session_id": "<uuid>"
}
```

**400 Bad Request** — `GenerateErrorResponse`:
```json
{
  "success": false,
  "errors": [{ "message": "...", "fields": ["..."] }],
  "minimum_thickness_mm": null
}
```

### `GET /api/download/step/{session_id}` and `/api/download/gltf/{session_id}`

- **200 OK** — binary body. STEP responds with `Content-Type: application/STEP`
  and `Content-Disposition: attachment`. glTF responds with
  `Content-Type: model/gltf+json` and `Content-Disposition: inline`.
- **404 Not Found** — session id unknown or file unreadable. Call `/api/generate`
  first; sessions live in server memory and don't survive a restart.

## Example: end-to-end generate + download

```sh
BASE=http://localhost:3030
PLATE='{"bolt_spacing":60,"bolt_size":"M10","bracket_height":40,"bracket_width":30,"material":"aluminum","pin_diameter":10,"pin_count":6,"plate_thickness":8,"expected_force_per_pin":500}'

# 1. Validate first (optional but cheap)
curl -sS -X POST "$BASE/api/validate" -H 'Content-Type: application/json' -d "$PLATE"

# 2. Generate; capture session id
SID=$(curl -sS -X POST "$BASE/api/generate" -H 'Content-Type: application/json' -d "$PLATE" | jq -r .session_id)

# 3. Download both files
curl -sS -o plate.step "$BASE/api/download/step/$SID"
curl -sS -o plate.gltf "$BASE/api/download/gltf/$SID"
```

## Common errors

- **`bolt_size` rejected.** The server accepts uppercase `M3`–`M12` only.
  `"m10"` or `"M14"` will fail.
- **`material` rejected.** Snake-case only: `stainless_steel`, not
  `StainlessSteel` or `stainlessSteel`.
- **`pin_count` out of range.** Must be 1–12 inclusive.
- **Stress errors.** Read `minimum_thickness_mm` and retry with a thicker plate,
  or reduce `expected_force_per_pin`.
- **404 on download.** The session id is from the `/api/generate` response, not
  a UUID you generate client-side. Restarting the server drops all sessions.

## Keeping this skill accurate

This file is kept in sync with the source via `scripts/check-api-sync.sh`.
When `crates/web/src/lib.rs` or `crates/domain/src/lib.rs` changes, the Claude
`PostToolUse` hook and CI both fail until this skill, the `CLAUDE.md` endpoint
table, and the `CLAUDE.md` example fetch are reviewed and `just update-api-hash`
is run.
