---
description: Verification before completion - ensure it's actually fixed/working
---

# Verification Before Completion Workflow

## Overview

**Purpose:** Prove that the implementation actually works before calling it "done."

"It should work" is not verification. "I verified it works" is.

## When to Use

- Before marking any task complete
- After fixing a bug
- After implementing a feature
- Before requesting review
- Before merging

## The Iron Rule

```
UNVERIFIED CODE IS NOT COMPLETE CODE
```

## Verification Levels

### Level 1: Unit Test Verification

**Required for:** All code changes

**Verify:**
- [ ] Test fails before fix (for bug fixes)
- [ ] Test passes after fix
- [ ] Edge cases covered
- [ ] Error paths tested

**How:**
```bash
# Run specific test
cd rust_ui && cargo test features::tdp

# Run with coverage (cargo-llvm-cov)
cd rust_ui && cargo llvm-cov

# Check coverage report - new code should be covered
```

### Level 2: Integration Verification

**Required for:** Multi-component changes, D-Bus contract changes, API changes

**Verify:**
- [ ] Components integrate correctly
- [ ] Data flows as expected
- [ ] Error handling works across boundaries
- [ ] No regression in existing integration

**How:**
```bash
# Run integration tests (packaging + daemon contracts)
cd rust_ui && cargo test --test packaging
cd rust_daemon && cargo test

# Or manual verification script
./scripts/verify-integration.sh
```

### Level 3: Manual Verification

**Required for:** UI changes, hardware interactions, complex workflows

**Verify:**
- [ ] UI renders correctly
- [ ] User interactions work
- [ ] Visual appearance matches design
- [ ] Performance acceptable

**How:**
```bash
# Run app locally with D-Bus tracing
make debug

# Test specific scenarios
# Document steps taken
```

### Level 4: Safety Verification (Critical)

**Required for:** Hardware control features (Constitution Principle I)

**Verify:**
- [ ] Safety limits enforced
- [ ] Invalid inputs rejected
- [ ] Edge cases (temp > 100°C, etc.) handled
- [ ] No dangerous defaults
- [ ] User confirmation for destructive actions

**How:**
```bash
# Test safety boundaries
# - Set TDP to 0 (should reject)
# - Set fan to 0% at 90°C (should enforce minimum)
# - Toggle dGPU (should warn user)

# Document safety test results
```

## Verification Checklist by Task Type

### Bug Fix

- [ ] Reproduce bug before fix (confirm bug exists)
- [ ] Apply fix
- [ ] Verify bug no longer reproducible
- [ ] Verify related functionality still works
- [ ] Add regression test

### New Feature

- [ ] Basic functionality works
- [ ] Edge cases handled
- [ ] Integration with existing features works
- [ ] Performance acceptable
- [ ] Documentation updated (if user-facing)

### Refactor

- [ ] Behavior unchanged (all tests pass)
- [ ] Performance not degraded
- [ ] Code is cleaner/readable
- [ ] No functional changes accidentally introduced

### Configuration Change

- [ ] Config loads correctly
- [ ] Invalid config handled gracefully
- [ ] Default values work
- [ ] Documentation updated

## Verification Documentation

**In plan file, for each task:**
```markdown
### Task 3: Implement TDP Write

**Verification:**
- [x] Unit test: `set_tdp(35)` against a fake `TdpService` → verifies saved settings
- [x] Integration: `features::tdp` → `Services` → asusd roundtrip (`make debug` trace)
- [x] Manual: Set TDP to 35W in UI, verified via `cat /sys/class/powercap/...`
- [x] Safety: Attempted 0W, 200W - both rejected with errors
```

## Anti-Patterns to Avoid

| Anti-Pattern | Why It's Wrong | Correct Approach |
|--------------|----------------|------------------|
| "It compiles, ship it" | Compilation != correctness | Run tests |
| "I tested the main path" | Edge cases break in production | Test edge cases |
| "The logic is obvious" | Obvious to you, not to the code | Add test |
| "I clicked around and it worked" | Not reproducible | Document test steps |
| "CI will catch it" | CI is slow feedback loop | Verify locally first |
| "It worked yesterday" | Things change, verify again | Re-verify |

## Red Flags

**Stop and verify more thoroughly if:**
- You didn't write any tests
- You didn't see it actually work
- You're skipping because "it's simple"
- You're in a hurry
- The change touches safety-critical code

## User Input

```text
$ARGUMENTS
```

If user indicates they want to verify something, guide them through appropriate verification level.

## Quick Verification Commands

**This project (Rust UI + Rust daemon):**

```bash
# Unit + integration tests of all crates
make test

# Manual (full stack)
make install       # build and install the daemon
make run           # build and run the UI
```

**Before claiming "done":**

```bash
# Full verification sequence
make test                                                # Unit + integration tests
cd rust_ui && cargo clippy --all-targets -- -D warnings  # Static analysis
make build-artifacts                                     # Release build verification
```

## Verification Culture

**Good:**
- "I verified it works by..."
- "Here's the test output..."
- "I tested edge cases X, Y, Z..."

**Bad:**
- "It should work..."
- "I'm pretty sure..."
- "I didn't test but..."

**Remember:** Unverified code is a liability. Verified code is an asset.
