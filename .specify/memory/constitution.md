<!--
Sync Impact Report (Version 1.2.0 → 2.0.0):
- Version change: 1.2.0 → 2.0.0 (MAJOR: principle III fundamentally changed)
- Bump rationale: the Flutter UI and flutter_rust_bridge FFI were replaced by a
  native Rust (egui) application `rust_ui` (2026-10-03). There is no FFI boundary
  any more; the UI talks to asusd/supergfxd/our daemon over D-Bus and links
  `rust_core` as a plain Rust library.
- Principles changed:
  II. Modular Service Architecture — adds the `rust_ui/src/features` service-trait layer
  III. Type-Safe FFI Boundary → Type-Safe Process Boundaries (D-Bus contracts)
  IV. Test-Before-Hardware — FFI/Dart test requirements replaced by Rust feature tests
  VI. Single Responsibility — Dart/Flutter widget rules replaced by egui screen rules
- Sections changed: Testing Standards (Flutter frontend → Rust UI), Test Execution,
  Input Validation (FFI parameters → D-Bus payloads)
- Principles unchanged: I, V, VII
- Templates requiring updates:
  ✅ .windsurf workflows/rules updated (sp.plan, sp.review, sp.finish, sp.execute,
     sp.tdd, error-handling, specify-rules)
  ⚠ plan-template.md / spec-template.md - unchanged, no Flutter references
- Follow-up TODOs:
  ⚠ Principle VI violations in rust_ui (files > 200 lines, P1 refactor):
    features/xg_diag/model.rs, features/fan/mod.rs, settings/store.rs,
    features/tdp.rs, features/gpu.rs, features/xg_diag/mod.rs, features/updates.rs,
    features/services.rs, backend/asusd.rs, features/fan/model.rs,
    features/cpu_boost.rs, ui/screens/fan.rs, ui/widgets.rs, ui/curve_editor.rs,
    settings/model.rs, ui/screens/tdp.rs, app.rs, ui/screens/xg_diag.rs
    (test modules features/*/tests.rs and settings/tests.rs also exceed the limit)
  ⚠ Principle III: `rust_core/src/services/performance.rs` still shells out to
    `dbus-send`; it is unused by `rust_ui` and should be removed or ported
- Last amended: 2026-10-03
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
- UI logic MUST live in `rust_ui/src/features/<domain>` and reach hardware only through
  a per-domain service trait (`PerformanceService`, `TdpService`, `FanService`, ...);
  production implementations live in `rust_ui/src/features/services.rs`

**Rationale:** Hardware features are orthogonal - a broken sensor module should not prevent performance mode changes. Independent modules enable parallel development and isolated testing. Mocking hardware dependencies becomes trivial when services are decoupled.

### III. Type-Safe Process Boundaries

The desktop application (`rust_ui`) and the privileged daemon (`rust_daemon`) are
separate processes. All communication between them, asusd and supergfxd MUST go
through typed D-Bus clients in `rust_ui/src/backend/`. FFI and shelling out to
`dbus-send` are prohibited.

**Mandatory Requirements:**
- Every D-Bus method/property used by the UI MUST be wrapped by a typed client method
  (explicit Rust argument and reply types matching the D-Bus signature)
- JSON payloads crossing D-Bus (fan engine config, CPU boost status, package update
  status) MUST be parsed into typed structs with explicit schema/version checks
- Client methods returning fallible operations MUST use `anyhow::Result<T>` with
  descriptive context
- Error messages MUST be user-facing (no internal implementation details leaked)
- D-Bus contract changes MUST update both the daemon and `rust_ui/src/backend/` in the same change

**Rationale:** Process boundaries replace the former language boundary. Typed clients
catch signature mismatches at compile time or in unit tests (e.g. curve struct
signature checks) instead of at runtime on real hardware.

### IV. Test-Before-Hardware

No code touching hardware interfaces (sysfs writes, USB HID, NVML calls) may be merged without:
1. Unit tests with mocked hardware interfaces (fake service traits)
2. Contract tests for D-Bus payload parsing and signatures
3. Manual validation on real hardware (documented in PR)

**Mandatory Test Coverage:**
- Every service module: >= 80% line coverage
- Every `rust_ui` feature operation: test against a fake service
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
- Rust source files MUST NOT exceed **200 lines**.
  Files approaching this limit MUST be split before adding new functionality.
  Services, hardware modules, and handlers MUST each live in their own file.
- egui screen code (`rust_ui/src/ui/`) MUST NOT contain business logic, D-Bus
  calls or settings writes; it renders state and dispatches `features` operations.
- A Rust function MUST do one thing: read OR write OR validate OR transform —
  never a combination in a single function body.
- Screens (`ui/screens/`) MUST only coordinate feature state and tasks and
  delegate reusable visual pieces to `ui/widgets.rs` or dedicated widget modules.
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
found the former `tdp_screen.dart` at 419 lines (4 distinct widgets) and a monolithic
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
- D-Bus replies and JSON payloads: typed deserialization, schema version checks, ranges checked in Rust
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

**Integration Tests** (`rust_ui/tests/`, `rust_daemon/tests/`):
- Verify D-Bus signatures and JSON payload contracts between UI and daemon
- Verify packaging and integration files (spec, PKGBUILD, D-Bus/polkit/systemd)
- Test async functions complete without deadlock

**Hardware Tests** (manual, documented in test reports):
- Performance mode changes reflected in `/sys` files
- Sensor readings match `sensors` utility output
- Fan curves applied correctly (measure actual RPM)
- Battery charge limit persists across reboots
- RGB/Anime Matrix displays correctly

### Rust UI Tests

**Feature Tests** (`rust_ui/src/features/**`):
- Every feature operation tested against fake service traits
- Saved-settings restore and mismatch paths covered
- Error states produce user-facing messages

**Widget Logic Tests** (`rust_ui/src/ui/**`):
- Pure editor/navigation logic (e.g. fan curve editing, gamepad commands) unit-tested
- Settings files stay compatible with previously written JSON

### Test Execution

Before every PR merge:
```bash
make test
cd rust_ui && cargo clippy --all-targets -- -D warnings
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

**Version**: 2.0.0 | **Ratified**: 2026-04-08 | **Last Amended**: 2026-10-03
