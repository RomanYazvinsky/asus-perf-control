<!--
Sync Impact Report (Version 1.1.0 → 1.2.0):
- Version change: 1.1.0 → 1.2.0 (MINOR: new principle + structural fix)
- Bump rationale:
  1. Principle VI moved to correct location in Core Principles
     (was misplaced inside Testing Standards section)
  2. Principle VII Plan-Review-Implement added: mandatory plan-first
     workflow with tmp/ artifacts and bug-loop discipline
- Principles fixed (location only, content unchanged):
  VI. Single Responsibility — moved from Testing Standards to Core Principles
- Principles added:
  VII. Plan-Review-Implement Discipline
- Principles unchanged:
  I. Hardware Safety First (NON-NEGOTIABLE)
  II. Modular Service Architecture
  III. Type-Safe FFI Boundary
  IV. Test-Before-Hardware
  V. Least Privilege Security Model
- Templates requiring updates:
  ⚠ plan-template.md - add VI, VII to Constitution Check section
  ⚠ spec-template.md - add file-size/responsibility constraints
  ⚠ tasks-template.md - add refactor task type for SRP violations
- Follow-up TODOs: none
- Last amended: 2026-04-10
-->

# ASUS Linux Control Tool Constitution

## Core Principles

### I. Hardware Safety First (NON-NEGOTIABLE)

Hardware control features MUST include safety validation before execution. This is non-negotiable because incorrect hardware control can cause permanent damage or safety hazards.

**Mandatory Requirements:**
- All fan control functions MUST validate minimum safe speeds (>= 30% PWM when temp > 80°C)
- Temperature thresholds MUST be enforced in code (CPU/GPU > 100°C triggers automatic fan response)
- Charge limits MUST be range-checked (0-100%) before sysfs write
- PWM values MUST be clamped to hardware limits (0-255)
- dGPU toggle operations MUST warn user of potential display disruption
- No hardware write operations without explicit user confirmation for destructive actions

**Rationale:** Laptop hardware lacks the physical failsafes of desktop systems. Software bugs in fan control can cause thermal damage. Battery overcharge control errors can degrade battery or cause safety issues. This principle protects users' hardware investment and physical safety.

### II. Modular Service Architecture

Backend MUST be organized into independent service modules, one per hardware feature domain (performance, sensors, battery, gpu, fans, rgb). Each module MUST be independently testable.

**Mandatory Structure:**
- `rust_core/src/services/{performance,sensors,battery,gpu,fans,rgb}.rs`
- Each service exposes a controller struct with `new()` constructor and feature methods
- Services MUST NOT directly call other services (use composition in api.rs instead)
- Each service MUST have corresponding unit tests in `rust_core/tests/`

**Rationale:** Hardware features are orthogonal - a broken sensor module should not prevent performance mode changes. Independent modules enable parallel development and isolated testing. Mocking hardware dependencies becomes trivial when services are decoupled.

### III. Type-Safe FFI Boundary

All communication between Flutter and Rust MUST use `flutter_rust_bridge` with generated bindings. Manual FFI is prohibited.

**Mandatory Requirements:**
- All public API functions in `rust_core/src/api.rs` MUST use `#[frb(sync)]` or async annotations
- All data structures crossing FFI MUST derive `Serialize` and be documented in `contracts/ffi-api.md`
- Codegen MUST be run after any API signature change before testing
- All FFI functions returning fallible operations MUST use `Result<T, String>` with descriptive errors
- Error messages MUST be user-facing (no internal implementation details leaked)

**Rationale:** Manual FFI is error-prone and breaks on Rust type changes. Generated bindings provide compile-time type safety across language boundaries. This eliminates an entire class of segfaults and memory safety bugs.

### IV. Test-Before-Hardware

No code touching hardware interfaces (sysfs writes, USB HID, NVML calls) may be merged without:
1. Unit tests with mocked hardware interfaces
2. Integration tests verifying FFI bridge functionality
3. Manual validation on real hardware (documented in PR)

**Mandatory Test Coverage:**
- Every service module: >= 80% line coverage
- Every FFI function: Integration test calling from Dart
- Every validation rule: Test with invalid inputs (expect errors)
- Every safety check: Test that dangerous operations are rejected

**Rationale:** Hardware control bugs can cause physical damage that unit tests on a development machine won't catch. However, mocked tests enable CI/CD and rapid iteration. The three-tier approach (unit → integration → hardware) balances safety with development velocity.

### V. Least Privilege Security Model

The application MUST NOT run as root. Hardware access MUST be granted via udev rules and polkit policies installed during setup.

**Mandatory Security Practices:**
- All sysfs writes MUST validate path (no directory traversal)
- All numeric inputs MUST be range-checked before hardware writes
- User configuration files MUST be stored in `~/.config/asus-linux-control/` (XDG spec)
- Installation script MUST document permission grants clearly
- USB HID access MUST use udev rules, not setuid binaries
- Polkit policies MUST be as narrow as possible (no blanket root access)

**Rationale:** Running desktop applications as root is a security anti-pattern. Linux provides privilege escalation mechanisms (udev, polkit) designed for this use case. Compromising the application should not grant full system access.

### VI. Single Responsibility

Every file, class, widget, and module MUST have exactly one reason to change.
God objects and mega-files are explicitly prohibited.

**Mandatory Rules:**
- Dart files MUST NOT exceed **150 lines** (generated files excluded).
  Files approaching this limit MUST be split before adding new functionality.
- Rust source files MUST NOT exceed **200 lines**.
  Services, hardware modules, and handlers MUST each live in their own file.
- A Flutter `Widget` class MUST NOT contain business logic, data fetching,
  or state management beyond coordinating its direct children.
- A Rust function MUST do one thing: read OR write OR validate OR transform —
  never a combination in a single function body.
- Screens (`screens/`) MUST only coordinate `BlocProvider`/`BlocBuilder` and
  delegate all visual sub-sections to dedicated widget files in `widgets/`.
- A service or repository class MUST cover exactly one feature domain
  (e.g., `BatteryService`, `TdpService`) — mixed-domain services are prohibited.
- D-Bus handlers in the Rust daemon MUST delegate hardware calls to the
  `hardware/` sub-modules; `main.rs` MUST contain no sysfs logic.

**Enforcement:**
- Code reviews MUST reject any PR that introduces a file exceeding the line limits
  without a documented, maintainer-approved exception.
- Refactoring existing violations is a P1 task and MUST be scheduled before
  adding new features to the affected file.

**Rationale:** Large files accumulate unrelated concerns, making bugs harder to
locate, tests harder to write, and changes higher-risk. The 2026-04-10 review
found `tdp_screen.dart` at 419 lines (4 distinct widgets) and a monolithic
`hardware.rs` — both split as immediate corrective actions. This principle
prevents recurrence.

### VII. Plan-Review-Implement Discipline

Every non-trivial change MUST follow a strict three-phase workflow:
**Plan → Review → Implement**. Implementation MUST NOT begin until the plan
has been reviewed. Bugs discovered during implementation MUST trigger a return
to the plan phase before any fixes are applied.

**Mandatory Workflow:**

1. **Plan phase** — Before writing any code:
   - Create an implementation plan file at `tmp/plan-<feature-slug>.md`.
   - The plan MUST include: goal, affected files, step-by-step approach,
     edge cases, rollback strategy, and open questions.
   - The plan MUST exist in `tmp/` before implementation begins.

2. **Review phase** — Before the first code change:
   - Re-read the plan and verify it is complete and consistent.
   - Check that the plan does not violate any constitution principle
     (especially I, II, VI).
   - If issues are found, update the plan; do NOT proceed to implementation
     with a known gap in the plan.

3. **Implement phase** — Only after review sign-off:
   - Follow the plan step by step. Do NOT deviate without updating the plan
     first.
   - After implementation, verify each plan step is satisfied.

4. **Bug loop** — When a bug is discovered post-implementation:
   - STOP. Do NOT apply a quick fix directly to code.
   - Return to `tmp/plan-<feature-slug>.md`, identify the root cause,
     and amend the plan with the correct approach.
   - Re-run the Review phase on the amended plan.
   - Only then implement the fix as described in the updated plan.
   - The final implementation MUST always match the plan; divergence is
     a process violation.

**Artifact Naming:**
- Plans: `tmp/plan-<slug>.md` (e.g., `tmp/plan-tdp-dbus.md`)
- Review reports: `tmp/review-<YYYY-MM-DD>.md`
- Both files MUST be kept for the lifetime of the feature branch.

**Enforcement:**
- No implementation work may begin without a corresponding `tmp/plan-*.md`.
- A bug fix that lacks an updated plan entry is a process violation and MUST
  be reverted and re-done with a proper plan amendment.

**Rationale:** Ad-hoc implementation without a written plan leads to scope
creep, missed edge cases, and hard-to-track bugs. The plan is a contract
between intent and execution. Returning to the plan on bugs forces root-cause
analysis instead of symptom patching — the primary source of regression in
this codebase.

## Security Requirements

### Input Validation

All external inputs MUST be validated before use:
- User configuration loaded from JSON: Schema validation with fallback to defaults
- Hardware sensor readings: Range checks (temps -50 to 120°C, RPM 0-10000)
- FFI parameters: Type enforced by bridge, ranges checked in Rust
- File paths: Canonicalize and verify prefix before access

### Permission Model

**Installation Phase** (one-time, requires sudo):
- Copy udev rules to `/etc/udev/rules.d/99-asus-control.rules`
- Copy polkit policies to `/etc/polkit-1/rules.d/50-asus-control.rules`
- Reload udev with `udevadm control --reload-rules && udevadm trigger`
- Document that reboot may be required for full effect

**Runtime Phase** (runs as normal user):
- Read/write sysfs interfaces granted by udev rules
- D-Bus access to UPower for battery monitoring (user session bus)
- USB HID access to Anime Matrix via udev rules
- Config file access restricted to user's home directory

### Error Handling

All operations returning `Result<T, String>` MUST provide actionable error messages:
- "Failed to set thermal policy: Permission denied. Run install.sh to set up udev rules."
- "Failed to read sensor: hwmon3 disconnected. Try restarting application."
- "Invalid fan curve: PWM too low at high temperature (safety check failed)."

Never expose:
- Stack traces to end users
- Internal file paths in production builds
- Raw OS error codes without context

## Testing Standards

### Rust Backend Tests

**Unit Tests** (`rust_core/tests/unit/`):
- Mock all hardware interfaces (use trait objects or feature flags)
- Test validation logic exhaustively (valid + invalid inputs)
- Test data structure serialization (config save/load)
- Run in CI without hardware dependencies

**Integration Tests** (`rust_core/tests/integration/`):
- Test FFI bridge by loading the dynamic library
- Verify generated bindings match API signatures
- Test async functions complete without deadlock
- Test stream functionality (sensor monitoring)

**Hardware Tests** (manual, documented in test reports):
- Performance mode changes reflected in `/sys` files
- Sensor readings match `sensors` utility output
- Fan curves applied correctly (measure actual RPM)
- Battery charge limit persists across reboots
- RGB/Anime Matrix displays correctly

### Flutter Frontend Tests

**Widget Tests** (`test/`):
- Every screen renders without error
- State changes update UI correctly
- Error states display user-friendly messages
- Navigation flows work correctly

**Integration Tests** (`integration_test/`):
- Complete user flow: Launch → change mode → verify setting persists
- Sensor monitoring stream updates UI in real-time
- Configuration save/load preserves all settings
- Error scenarios display dialogs and don't crash

### Test Execution

Before every PR merge:
```bash
cd rust_core && cargo test && cargo clippy -- -D warnings
cd .. && flutter test && flutter analyze
```

Before every release:

- Run full hardware test suite on 2+ laptop models
- Document tested models and kernel versions in release notes
- Verify udev rules work on fresh installation

## Governance

This constitution is the authoritative source for project development standards. It supersedes informal practices, external tool defaults, and individual preferences.

**Amendment Process:**
1. Propose changes via issue or discussion with rationale
2. Amendments affecting safety (Principle I) require unanimous maintainer approval
3. Other amendments require majority maintainer approval
4. All amendments MUST increment version semantically:
   - MAJOR: Principle removed or fundamentally changed
   - MINOR: New principle added or section expanded
   - PATCH: Clarifications, typos, non-semantic wording
5. Update templates (plan-template.md, spec-template.md, tasks-template.md) for consistency
6. Document amendment in Sync Impact Report at file top

**Compliance Verification:**
- All PRs MUST include checklist verifying relevant principles
- Code reviews MUST verify safety validations are present (Principle I)
- Releases MUST include hardware test report (Principle IV)
- Security issues MUST be reported privately and fixed before disclosure

**Complexity Justification:**
- Any violation of principles MUST be justified in design doc and approved by maintainers
- Technical debt intentionally incurred MUST have mitigation plan with timeline
- Workarounds for hardware limitations MUST be documented with references

**Version**: 1.2.0 | **Ratified**: 2026-04-08 | **Last Amended**: 2026-04-10
