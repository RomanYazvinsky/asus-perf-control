---
description: Write comprehensive implementation plans before touching code
---

# Writing Plans Workflow

## Overview

Write comprehensive implementation plans assuming the engineer has **zero context** for our codebase and questionable taste. Document everything they need to know.

**Save plans to:** `tmp/plan-<feature-slug>.md` (aligns with Constitution Principle VII)

## Core Principles

- **DRY** - Don't Repeat Yourself
- **YAGNI** - You Aren't Gonna Need It  
- **TDD** - Test-Driven Development
- **Frequent commits**
- **Small files** (per Constitution Principle VI)

## User Input

```text
$ARGUMENTS
```

If user provided feature/requirements, create plan for that. Otherwise, ask what feature to plan.

## Scope Check

If the spec covers multiple independent subsystems:
- Break into separate plans — one per subsystem
- Each plan should produce working, testable software on its own

## File Structure Mapping

**Before defining tasks, map out which files will be created or modified:**

- Design units with clear boundaries and well-defined interfaces
- Each file should have one clear responsibility
- Files that change together should live together
- Split by responsibility, not by technical layer

**Constitution VI Compliance:**
- Rust files: max 200 lines
- One feature per service trait (`rust_ui/src/features/`)
- Screens render state and delegate to widgets; no business logic in `ui/`

## Plan Document Header

**Every plan MUST start with this header:**

```markdown
# [Feature Name] Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use /sp.execute to implement this plan task-by-task.

**Goal:** [One sentence describing what this builds]

**Architecture:** [2-3 sentences about approach]

**Tech Stack:** [Key technologies/libraries]

**Affected Files:**
- Create: `path/to/new/file.rs`
- Modify: `path/to/existing.rs:123-145`

---
```

## Bite-Sized Task Granularity

**Each step is one action (2-5 minutes):**

```markdown
### Task N: [Component Name]

**Files:**
- Create: `exact/path/to/file.py`
- Modify: `exact/path/to/existing.py:123-145`
- Test: `tests/exact/path/to/test.py`

- [ ] **Step 1: Write the failing test**

```python
def test_specific_behavior():
    result = function(input)
    assert result == expected
```

- [ ] **Step 2: Run test to verify it fails**

Run: `pytest tests/path/test.py::test_name -v`
Expected: FAIL with "function not defined"

- [ ] **Step 3: Write minimal implementation**

```python
def function(input):
    return expected
```

- [ ] **Step 4: Run test to verify it passes**

Run: `pytest tests/path/test.py::test_name -v`
Expected: PASS

- [ ] **Step 5: Commit**
```

## Task Ordering Principles

1. **Tests before implementation** (TDD)
2. **Lower layers first** (bottom-up)
3. **Vertical slice for integration** (one complete flow)
4. **Refactoring last** (green first, then clean)

## Testing Strategy Section

Every plan needs explicit testing strategy:

```markdown
## Testing Strategy

- Unit tests: [what functions, what edge cases]
- Integration tests: [which component interactions]
- Manual verification: [if needed]
- Performance considerations: [if applicable]
```

## Constitution Compliance Check

Before finalizing plan, verify:

- [ ] **Principle I (Hardware Safety):** Any hardware controls have safety validation?
- [ ] **Principle VI (Single Responsibility):** No file exceeds line limits, no god objects?
- [ ] **Principle VII (Plan-Review-Implement):** Plan saved to `tmp/plan-*.md`?
- [ ] **Principle IV (Test-Before-Hardware):** Tests written before hardware-touching code?

## Post-Planning

After plan is complete:
1. Save to `tmp/plan-<slug>.md`
2. User reviews and approves
3. Use `/sp.execute` to implement task-by-task

## Example Task

```markdown
### Task 1: Create TDP Service

**Files:**
- Modify: `rust_ui/src/features/tdp.rs`
- Test: `rust_ui/src/features/tdp.rs` (`#[cfg(test)] mod tests`)

- [ ] **Step 1: Write failing test for load()**

Uses a fake `TdpService` returning a known config, verifies the loaded state.

```rust
#[tokio::test]
async fn load_returns_hardware_limits() {
    // Arrange
    let service = fake(config(35, 45, true, 0));
    // Act
    let state = load(&service, &store(None)).await;
    // Assert
    assert!(matches!(state, TdpState::Loaded(l) if l.sustained_w == 35));
}
```

- [ ] **Step 2: Run test - expect FAIL**
- [ ] **Step 3: Implement readTdp() minimal version**
- [ ] **Step 4: Run test - expect PASS**
- [ ] **Step 5: Commit: "Add TDP service with read support"**
```
