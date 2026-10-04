---
description: Test-Driven Development - RED-GREEN-REFACTOR cycle
---

# Test-Driven Development Workflow

## Overview

Write the test first. Watch it fail. Write minimal code to pass.

**Core principle:** If you didn't watch the test fail, you don't know if it tests the right thing.

```
NO PRODUCTION CODE WITHOUT A FAILING TEST FIRST
```

## When to Use

**Always:**
- New features
- Bug fixes
- Refactoring
- Behavior changes

**Exceptions (ask your human partner):**
- Throwaway prototypes
- Generated code
- Configuration files

Thinking "skip TDD just this once"? Stop. That's rationalization.

## User Input

```text
$ARGUMENTS
```

If user provided feature to implement, start TDD cycle. Otherwise, ask what to build.

## Red-Green-Refactor Cycle

### RED - Write Failing Test

Write one minimal test showing what should happen.

**Good Test:**
- Clear name describing behavior
- Tests real behavior, not implementation
- One thing per test
- Readable without looking at implementation

```rust
#[tokio::test]
async fn retries_failed_operations_3_times() {
    let mut attempts = 0;
    let result = retry_operation(|| {
        attempts += 1;
        if attempts < 3 { Err("boom") } else { Ok("success") }
    })
    .await;

    assert_eq!(result, Ok("success"));
    assert_eq!(attempts, 3);
}
```

**Bad Test:**
- Vague name
- Mocks implementation details
- Tests multiple things
- Unclear what it's verifying

**After writing test:**
- Run it immediately
- Verify it fails for the RIGHT REASON
- Wrong error = fix test first

### GREEN - Minimal Implementation

Write the simplest code that makes the test pass.

**Rules:**
- No design thinking yet
- No "this should be cleaner"
- Just make it green
- Hardcode if needed (seriously)
- Copy-paste if it works

**Constraints:**
- Only implement what's tested
- No "while I'm here" features
- No refactoring yet
- Ugly is fine

```rust
// Ugly but GREEN is fine
async fn retry_operation<T, E>(mut f: impl FnMut() -> Result<T, E>) -> Result<T, E> {
    f().or_else(|_| f()).or_else(|_| f())
}
```

**After implementing:**
- Run all tests
- Verify new test passes
- Verify existing tests still pass

### REFACTOR - Clean Up

Now that tests pass, make it clean.

**Safe Refactoring (with passing tests):**
- Rename variables
- Extract functions
- Remove duplication
- Improve naming
- Simplify conditionals
- Add types/clarity

**Golden Rule:**
```
Tests must stay green throughout refactoring
```

If tests fail during refactor:
1. You broke something
2. Revert the change
3. Smaller refactor steps

**After each micro-refactor:**
- Run tests
- If green, commit the refactor
- If red, revert and try smaller step

## Test Quality Checklist

- [ ] Test fails first (RED confirmed)
- [ ] Test passes with minimal code (GREEN confirmed)
- [ ] Test name describes behavior, not implementation
- [ ] Test reads like documentation
- [ ] One concept per test
- [ ] No implementation details in test
- [ ] Edge cases covered separately
- [ ] Test doesn't mirror implementation structure

## Anti-Patterns

| Anti-Pattern | Why It's Wrong | Fix |
|--------------|----------------|-----|
| Mocking implementation | Test breaks on refactor | Mock at boundaries only |
| Testing getters/setters | Tests language, not logic | Delete these tests |
| Giant test with 20 asserts | Fails unclear why | Split into focused tests |
| Testing private methods | Tests internals, not behavior | Test via public API |
| `test('it works', ...)` | No idea what "it" or "works" means | Descriptive names |
| Tests without asserts | Silent pass on broken code | Always assert expected outcome |

## Project-Specific TDD

**Rust UI features (this project):**

```rust
// Test screen logic against a fake service trait
#[tokio::test]
async fn restores_saved_charge_limit() {
    // Arrange: hardware reports 100 %, settings say 80 %
    let service = fake(100, true);

    // Act
    let state = check_support_and_load(&service, &store(Some(80))).await;

    // Assert
    assert_eq!(state, BatteryState::Loaded { charge_limit: 80 });
}
```

**Rust Daemon:**

```rust
#[test]
fn test_fan_curve_validation() {
    // Arrange
    let curve = FanCurve::new(vec![(30, 0), (50, 50), (70, 100)]);
    
    // Act & Assert
    assert!(curve.validate().is_ok());
    
    // Invalid: decreasing
    let bad = FanCurve::new(vec![(30, 50), (50, 30)]);
    assert!(bad.validate().is_err());
}
```

## Full TDD Session Example

```
1. Write test for "user can set TDP to 35W"
2. Run test → FAIL (TdpService doesn't exist)
3. Create TdpService with stub method → FAIL (wrong return type)
4. Fix return type → FAIL (returns null not 35)
5. Hardcode return 35 → GREEN!
6. Refactor: extract validation, add types
7. Write test for "TDP clamps to safe range"
8. Run test → FAIL (no clamping)
9. Add clamping → GREEN!
10. Refactor: extract Range class
11. Commit
```

## Integration with Plan

When following a written plan:

1. Read current task from plan
2. Write failing test for that task
3. RED → GREEN → REFACTOR
4. Check off task in plan
5. Commit
6. Next task
