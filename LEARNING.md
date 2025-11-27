# Actuator Plate Configurator - Learning Goals & Success Criteria

## Project Purpose

Validate whether I can build a **web → parametric CAD → quote pipeline** that works in real-time. This is a technical research project, not a business validation. The goal is to learn about the tooling and workflow for automated custom manufacturing.

## Why Plates?

- Simple geometry (2D shapes, holes, mounting patterns)
- Fast iteration on the technical pipeline
- Clear success/failure criteria
- Can actually fulfill orders if needed (via SendCutSend/Xometry)
- Tests the hard parts (parametric generation, cost modeling) without CAD complexity

## Core Technical Questions

### 1. Pipeline Architecture
**Q: Can I build a web → CAD → file pipeline that completes in <10 seconds?**

Success criteria:
- User submits form
- Gets viewable 3D model + downloadable STEP file
- Total time < 10 seconds

Measurements:
- Response time breakdown (network, computation, file generation)
- File sizes
- Error rates

Learning outcomes:
- Where are the bottlenecks?
- What's the practical latency for user-facing parametric CAD?

### 2. CAD Tool Selection
**Q: Which CAD system actually works for server-side parametric generation?**

Options to evaluate:
- OnShape API (cloud-based, REST API)
- FreeCAD + Python (open source, scriptable)
- OpenSCAD (code-based CAD)
- CadQuery (Python library)
- `truck` crate (pure Rust, experimental)

Success criteria:
- Can generate valid STEP/STL files programmatically
- Works reliably in server environment
- Handles concurrent requests

Learning outcomes:
- Licensing constraints
- API reliability and documentation quality
- Geometry capabilities and limitations
- Cost at scale
- Operational complexity (deployment, updates, debugging)

### 3. Parameter → Geometry Mapping
**Q: How do I translate user inputs into valid CAD constraints?**

Success criteria:
- Can define plates with:
  - Dimensions (length, width, thickness)
  - Hole patterns (positions, diameters)
  - Material selection
  - Mounting styles

Edge cases to test:
- Holes too close to edges
- Holes too close to each other
- Invalid thickness for given dimensions
- Impossible geometries
- Standard bolt patterns (M3, M4, M5, etc.)

Learning outcomes:
- What breaks the constraint system?
- How do I provide good error messages?
- What validation happens client-side vs server-side?

### 4. Manufacturing Validation
**Q: Can I programmatically validate that generated designs are manufacturable?**

Validation rules to implement:
- Minimum feature sizes
- Edge distances for holes
- Material availability
- Tool access considerations
- Cost-prohibitive dimensions

Success criteria:
- Catch common manufacturing errors before quote
- Provide actionable error messages
- Rules are tunable (different for laser vs waterjet vs CNC)

Learning outcomes:
- What rules actually matter for real manufacturing?
- How do I encode domain knowledge in validation logic?
- Can this be done without deep manufacturing expertise?

### 5. Cost Modeling
**Q: Can I calculate accurate quotes from geometry alone?**

Formula components:
- Material cost: area × thickness × material price per unit
- Cutting cost: perimeter length + hole count
- Finishing: powder coat, anodizing, etc.
- Setup/handling fees
- Quantity discounts

Success criteria:
- Formula produces quotes within 20% of real quotes
- Can compare against SendCutSend/Xometry for validation

Validation approach:
- Generate 5-10 test designs
- Get real quotes from services
- Tune cost model to match
- Document assumptions and limitations

Learning outcomes:
- How much complexity is needed for "good enough" quotes?
- What are the main cost drivers?
- Where does the model break down?

### 6. Rust Web Stack
**Q: What's the right Rust web framework for this application?**

Options to evaluate:
- Axum (tokio-based, minimal) -- `had experience with this in another project, went with it without evaluating other options`
- Actix-web (mature, performant)
- Rocket (ergonomic, batteries-included)

Success criteria:
- Handle form submissions
- Run CAD generation asynchronously
- Serve files for download
- Handle errors gracefully

Learning outcomes:
- Async runtime patterns in practice
- Error handling across async boundaries
- File streaming and cleanup
- Deployment story

### 7. Validation Architecture
**Q: Should validation be client-side (WASM), server-side (WebSocket), or both?**

**Option A: Server-only (baseline)**
- Simple HTTP POST on submit
- All validation on server
- No real-time feedback

**Option B: WASM client-side + server validation**
- Validation crate compiles to native and WASM
- Instant feedback in browser (<100ms)
- Server validates again (trust boundary)

**Option C: WebSocket real-time**
- Persistent connection to server
- Server validates on every input change
- Round-trip latency (50-200ms)

Success criteria for WASM approach:
- Same validation crate works in both environments
- Client validation provides instant feedback
- Server catches malicious/tampered inputs
- WASM bundle loads quickly (<500KB)

Learning outcomes:
- Is WASM practical for this use case?
- What's the debugging experience like?
- How do you handle validation logic that can't run in WASM?
- What's the performance difference?

## Crate Architecture (if using WASM)

```
actuator-plate/
├── plate-geometry/          # Core domain logic (no_std compatible)
│   ├── validation.rs        # Manufacturing constraint checks
│   ├── constraints.rs       # Geometric rules
│   ├── pricing.rs           # Cost calculation
│   └── types.rs             # PlateConfig, ValidationError, etc.
│
├── plate-web/               # Web server (Axum/Actix)
│   ├── handlers.rs          # HTTP routes
│   └── cad.rs               # CAD generation (calls external tools)
│
└── plate-wasm/              # Browser-side validation
    └── lib.rs               # wasm-bindgen wrapper
```

## Project Success Criteria

### Minimum Success (2-3 weeks of focused work)
- [ ] Web form with 5-7 parameters (length, width, thickness, hole pattern, material)
- [ ] Server generates valid STEP file from parameters
- [ ] User can download file and view in free CAD viewer (FreeCAD, OnShape)
- [ ] Cost calculation within 20% of real quote from manufacturing service
- [ ] At least 10 validation rules with clear, actionable error messages
- [ ] Documentation of learnings for each core question

### Stretch Goals (if momentum continues)
- [ ] Real-time 3D preview in browser (Three.js)
- [ ] WASM validation with instant feedback
- [ ] Auto-validation with visual feedback (highlight problem areas)
- [ ] Integration with SendCutSend or Xometry API for real quotes
- [ ] Property tests for validation invariants (`proptest`)
- [ ] Deployment to production hosting

## Decision Points

### After Minimum Success, answer:

1. **Enjoyment test**: Was this engaging enough to continue?
   - Did I stay focused or was it drudgery?
   - Do I want to scale this to more complex geometry?

2. **Technical feasibility**: Do these capabilities scale to actuators?
   - Can the CAD tool handle assemblies and moving parts?
   - Is the cost model extendable to more complex components?
   - What breaks when going from 2D plates to 3D assemblies?

3. **Business viability**: Is this worth pursuing commercially?
   - Would customers actually use this?
   - Is the cost model accurate enough to trust?
   - What's the path from plates to full actuators?

### Pivot Options Based on Learnings

**If technical works but plates are boring:**
- Apply same pipeline to actuator components
- Explore brackets, mounts, or other simple mechanical parts
- Move toward "nervous system" components (cable routing, connectors)

**If technical works great and I'm engaged:**
- Scale to full actuator configurator (more complex geometry)
- Add payment processing and actually fulfill orders
- Explore other custom manufacturing applications

**If technical is too slow/clunky:**
- Different CAD tool might work better
- Simplify geometry constraints
- Accept higher latency and set expectations differently

**If technical works but I hate building it:**
- Document learnings and move on (no shame in this)
- Consider partnering with someone who enjoys this work
- Focus energy on different aspects of actuator business

## Anti-Abandonment Strategy

**Commit to writing a blog post about findings regardless of completion status.**

This ensures that even if I stop at 60%, I've:
- Extracted the learnings
- Made it useful for others
- Have clear documentation of what I learned
- Maintained momentum on technical writing

Topics for blog post:
- "Evaluating CAD APIs for Parametric Generation"
- "Building a Custom Manufacturing Quote Engine"
- "Rust + WASM for Real-time Validation"
- "What I Learned Building an Actuator Plate Configurator"

## Timeline & Reality Check

**Week 1-2**: CAD tool research + hello world (parameters → file)
**Week 3-4**: Web interface + cost model
**Week 5**: Testing + documentation of learnings

**Engagement checkpoints:**
- Week 2: Still interested? (Good sign if yes)
- Week 3: Feeling momentum or drag? (Valid data either way)
- Week 5: Ready to continue or ready to document and move on?

**Expected timeline with full-time job + three kids:**
- 5-10 hours/week available
- Best case: 5 weeks to minimum success
- Realistic: 8-10 weeks accounting for holidays and life
- Acceptable: Stop at any checkpoint and document learnings

## What This Unlocks (Future Applications)

**Immediate next steps if successful:**
- Scale to actuator configurator (full assemblies)
- Apply to other custom mechanical components
- Explore actuator manufacturing as business

**Technical patterns that transfer:**
- Parametric design automation
- Web → computation → file download pipelines
- Real-time validation in browser
- Cost estimation from CAD geometry

**Skills that compound:**
- Rust web frameworks in practice
- CAD API integration
- Manufacturing domain knowledge
- WASM for computation-heavy web apps

## Notes

- This is a learning project, not a business validation
- No need for customer acquisition, marketing, or payment processing at minimum success
- Focus is on proving the technical pipeline, not market demand
- Success = answering the core questions, not building a company
