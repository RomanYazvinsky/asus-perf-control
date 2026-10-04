# Full-Project Quality and Safety Design

**Date:** 2026-09-26

## Purpose

Raise the quality baseline of the complete ASUS Linux Control repository without
changing user-visible hardware-control behavior. The work covers the Flutter
application, `rust_core`, the privileged Rust daemon, D-Bus and Polkit
boundaries, installation scripts, systemd and udev integration, and native
packaging.

The current uncommitted fan-screen work is user-owned. Quality changes must be
applied on top of it and must not replace or revert it.

## Baseline

- Flutter: 236 tests pass. `flutter analyze` exits non-zero because of six
  informational lint findings, including duplicates under `dist/`.
- Rust: 15 `rust_core` tests and 155 `rust_daemon` tests pass.
- `rust_core` fails formatting and strict Clippy.
- `rust_daemon` passes formatting but fails strict Clippy.
- Install tests pass 28/28 and tracked shell files pass syntax checks.
- No repository CI configuration exists.

Passing tests do not make the current implementation quality-complete. The
baseline contains unsafe boundary decoding, handwritten panic paths, unchecked
numeric conversions, oversized modules, duplicated Rust test imports, unsafe
device permissions, and incomplete package metadata.

## Goals

1. Make malformed external data fail as a typed, actionable error instead of a
   runtime cast failure, null assertion, panic, or silent fallback.
2. Remove handwritten Rust `unsafe` blocks and production `unwrap`/`expect`
   calls.
3. Use checked numeric conversions wherever a value can narrow, overflow, or
   cross a process boundary.
4. Split large files along stable responsibilities while preserving behavior
   and public contracts.
5. Apply least privilege to D-Bus, Polkit, udev, and systemd integration.
6. Make one local command verify the same quality gates across every maintained
   layer.
7. Make native package inputs deterministic, compact, and explicit about their
   runtime dependencies.

## Non-Goals

- No visual redesign or new hardware feature is part of this work.
- No manual edits to Flutter Rust Bridge generated files.
- No dependency upgrade solely to obtain newer versions. A dependency changes
  only when required for a clean supported toolchain or to remove a concrete
  safety problem, and it receives its own compatibility verification.
- No invention of a software licence. Until the owner explicitly confirms the
  licence, release validation must report the missing `LICENSE` file and package
  publication remains blocked.
- No cleanup of historical output already under `dist/` unless the user asks to
  remove it. Quality tools must ignore it as generated output.

## Safety Rules

### Dart

- Handwritten code must not cast untrusted JSON, D-Bus values, platform data,
  or plugin results with an unchecked `as` expression.
- External values are decoded by small typed readers that validate the value,
  field name, expected type, range, and nullability before returning.
- `dynamic` is permitted only at an external decoding boundary and must not
  escape that boundary.
- Non-null assertions are removed from production paths. A value proven by a
  preceding branch is promoted or bound to a local non-null variable.
- Framework object lookups such as `findRenderObject()` use checked pattern
  matching and produce a controlled no-op or typed error when the expected
  object is unavailable.
- `analysis_options.yaml` enables strict casts, strict inference, and strict raw
  types for maintained source and tests. Generated bindings and build outputs
  are excluded explicitly.

### Rust

- Handwritten production modules contain no `unsafe` blocks. The current libc
  `geteuid` and `poll` calls move to safe `nix` APIs with narrowly enabled crate
  features.
- Handwritten production code contains no `unwrap`, `expect`, `panic!`,
  `todo!`, or `unimplemented!` on runtime paths.
- Narrowing conversions use `TryFrom` or a validated newtype. Sentinel values
  such as `-1` do not cross into unsigned code.
- Poisoned locks and clocks before the Unix epoch return contextual errors;
  they do not terminate a daemon request.
- Generated FRB code is the sole exception to handwritten-code rules. It is
  reproducible from pinned inputs, excluded narrowly from relevant lints, and
  never edited to hide a generator problem.

### Shell and system integration

- Scripts quote variable expansions, use explicit validated targets for
  destructive operations, and never grant world-writable device access.
- Privileged writes remain behind daemon methods and Polkit authorization.
- Generated artifacts never become source-package inputs accidentally.

## Workstream 1: Security and Quality Gates

This workstream is first because later refactors need a trustworthy safety net.

### Device permissions

- Remove udev assignments that write the string `0666` into sysfs attributes.
- Remove the world-writable ASUS USB rule.
- Restrict the hotkey path to the required ASUS `hidraw` nodes using mode
  `0660` and the existing controlled access group; no generic ASUS USB device
  becomes writable.
- Fan and thermal writes remain privileged daemon responsibilities rather than
  direct GUI sysfs access.
- Add semantic tests that reject `MODE="0666"`, sysfs `ATTR=..."0666"`
  assignments, and broader device grants.

### Authorization

- Represent the Polkit caller as a `system-bus-name` subject using the unique
  D-Bus sender name. Do not construct a `unix-process` subject with an unknown
  start time.
- All mutating hardware actions use `auth_admin_keep` for an active session and
  `auth_admin` otherwise. Read-only methods remain callable without
  authorization.
- Tests keep D-Bus member allowlists and Polkit action IDs synchronized with
  the Rust interface.

### Unified verification

- Add a non-destructive `make check` entry point covering:
  - Dart formatting, analysis, and the full Flutter test suite;
  - Rust formatting, strict Clippy, and all tests for both crates;
  - shell syntax and available static shell analysis;
  - D-Bus, Polkit, udev, desktop, AppStream, systemd, RPM, and Arch metadata
    validation when the corresponding host tool is installed.
- Missing optional host validators are reported as skipped. Core Dart and Rust
  checks are mandatory and fail the command when unavailable or unsuccessful.
- Generated directories (`build`, `dist`, `.dart_tool`, Cargo `target`) are not
  scanned as maintained source.

## Workstream 2: Flutter Type Safety and Decomposition

### Boundary adapters

- Introduce shared checked JSON readers for required/optional strings, booleans,
  integers, lists, and string-keyed maps.
- Introduce checked D-Bus response readers for expected cardinality, variants,
  arrays, structs, and scalar types.
- Package update, settings, CPU boost, asusd fan, supergfx, and XG diagnostic
  decoding use these adapters and return domain-specific exceptions containing
  the field or method name.

### Stable module boundaries

- Split application settings into domain models, JSON codecs, migrations, and
  persistence. Model `copyWith` semantics remain null-safe without an untyped
  sentinel leaking through public APIs.
- Split XG fan diagnostics into ports, serializable models/codecs, session
  storage, sweep orchestration, and diagnosis/classification. Hardware access
  and pure diagnosis logic remain independently testable.
- Extract fan-curve policy and persisted-settings mapping from `FanBloc`.
  `FanBloc` remains responsible for event sequencing and state emission only.
- Split large screens into focused private or feature-local widgets without
  changing navigation, keys used by tests, or accessibility behavior.
- Consolidate the own-daemon D-Bus constants and invocation/error mapping so
  each feature client describes only its methods and payloads.

### Flutter acceptance criteria

- `flutter analyze` is clean against maintained source and tests.
- All existing tests pass, including the user-owned fan-screen tests.
- New tests cover malformed JSON, wrong D-Bus types/cardinality, absent render
  objects, and nullable states without runtime type errors.
- Widget tests inject service fakes and do not emit avoidable native-bridge
  initialization errors.

## Workstream 3: Rust Core and Daemon Decomposition

### Daemon crate shape

- Add `rust_daemon/src/lib.rs` exporting the production modules required by
  binaries and integration tests.
- `main.rs` becomes composition and lifecycle wiring: logging, signal setup,
  dependency construction, D-Bus registration, task startup, and shutdown.
- Integration tests import `asus_control_daemon` normally. All test and source
  `#[path = ...]` imports of production modules are removed.

### Daemon responsibilities

- Move the D-Bus interface into a dedicated module and divide handlers by
  hardware, fan engine, package updates, and diagnostics.
- Centralize sender extraction, authorization, error translation, and JSON
  serialization to remove repeated boilerplate.
- Split ROG hotkey status/storage, startup resolution, monitor supervision, and
  shutdown restoration into focused modules.
- Split CPU boost discovery, apply policy, persistence, and baseline tracking.
- Preserve the package manager executable/argument allowlist and expose command
  construction as pure, testable data.

### `rust_core`

- Keep generated bindings isolated from handwritten API and service modules.
- Replace sentinel-based TDP updates with typed optional fields.
- Convert timestamps and hardware values with checked conversions and explicit
  range errors.
- Remove current formatting and Clippy findings in handwritten code. Resolve
  the pinned FRB `unexpected_cfgs` issue through a compatible regeneration or a
  generator-scoped lint configuration, never a repository-wide warning
  suppression.

### Rust acceptance criteria

- Formatting, strict Clippy, and all tests pass for both crates.
- Test totals represent distinct tests rather than modules recompiled through
  multiple `#[path]` imports.
- No handwritten production `unsafe`, runtime panic macro, `unwrap`, or
  `expect` remains.
- Startup, shutdown, lock poisoning, invalid payloads, and failed hardware
  operations have explicit test coverage.

## Workstream 4: Packaging and Release Integrity

- Declare the actual GUI and daemon runtime dependencies for RPM and Arch.
- Replace the fixed `/usr/local/bin/ryzenadj` assumption with deterministic
  executable discovery and a clear unsupported/error state. Packages either
  declare the selected provider or disable the feature explicitly when no
  provider exists.
- Build a staging tree containing only the release bundle, daemon binaries,
  metadata, and required source documents. Cargo targets and unrelated Flutter
  build trees never enter the source archive.
- Clean each package-output staging directory before building so versions do
  not mix.
- Resolve home directories through the account database consistently; do not
  construct `/home/$USER` paths.
- Add package checks for dependencies, version agreement, file ownership,
  licence presence, archive contents, and service/config installation paths.
- Add conservative systemd hardening that is proven compatible with required
  sysfs, state-directory, D-Bus, hotkey, and package-manager operations. The
  service must fail closed when a required privilege is unavailable.
- Correct desktop and AppStream metadata warnings.

## Sequencing and Change Isolation

Each workstream receives a separate implementation plan and is delivered in
small, testable tasks. The sequence is:

1. Security and quality gates.
2. Flutter boundary safety and decomposition.
3. Rust crate structure, safety, and decomposition.
4. Packaging and release integrity.

Cross-layer contract changes are introduced producer-first with compatibility
tests, then consumed by the adjacent layer before obsolete code is removed.
Behavioral refactors keep existing tests green; safety defects receive a failing
regression test before their fix.

Only files belonging to the active task are staged and committed. Existing
unrelated modifications and generated output remain untouched.

## Completion Criteria

The project-wide effort is complete only when:

1. `make check` passes on maintained source with no warnings hidden by broad
   suppression.
2. All Flutter and Rust tests pass from a clean checkout.
3. Handwritten external-boundary decoding is checked and produces typed errors.
4. Handwritten Rust production code contains no unsafe block or panic path.
5. Privileged operations are least-privilege and covered by semantic policy
   tests.
6. Large modules named in this design have been split by responsibility and
   their public behavior remains covered.
7. Package staging is deterministic and does not include build caches.
8. Release validation reports the licence state accurately and refuses to
   publish an inconsistent package.

