---
description: Execute plans task-by-task with subagent-driven development
---

# Executing Plans Workflow

## Overview

**Purpose:** Implement written plans task-by-task with checkpoint verification.

This workflow follows the **subagent-driven-development** pattern: break work into discrete tasks, complete each one fully before proceeding, verify at each checkpoint.

## When to Use

- Following a written plan (`tmp/plan-*.md`)
- Multi-step implementation
- Complex features requiring verification between steps
- Any work where "done" has multiple components

## Prerequisites

**MUST have:**
- Written plan at `tmp/plan-<feature>.md`
- Plan reviewed and approved
- Clear understanding of current task

## User Input

```text
$ARGUMENTS
```

If user specifies which task to work on, start there. Otherwise, identify the next uncompleted task from the plan.

## The Execution Cycle

### 1. Read Current Plan

Load `tmp/plan-<feature>.md` and identify:
- Which tasks are already complete (checked off)
- Which task is next
- Dependencies between tasks

### 2. Understand the Current Task

**For the current task, identify:**
- What files need to be created/modified
- What tests need to be written
- What the acceptance criteria are
- Any specific implementation notes

### 3. Execute Task

**Follow TDD (RED-GREEN-REFACTOR):**

```
RED:      Write failing test
GREEN:    Minimal implementation
REFACTOR: Clean up
```

**Each step must be complete before next:**
- Test MUST fail for right reason before implementing
- Implementation MUST pass all tests before refactoring
- Refactor MUST keep all tests green

### 4. Verify Task Completion

**Check against plan:**
- All steps in task checked off?
- Tests pass?
- Code follows project conventions?
- Constitution compliance verified?

**Specific Constitution Checks:**
- [ ] Principle I (Hardware Safety): Safety validation in place?
- [ ] Principle VI (Single Responsibility): File under 200 lines?
- [ ] Principle IV (Test-Before-Hardware): Tests before hardware code?

### 5. Update Plan

**Mark task complete:**
```markdown
- [x] **Step 1: Write the failing test** ✓
  - Test written: `rust_ui/src/features/tdp.rs` (tests module)
  - Verified: fails with "function not defined"
```

### 6. Next Task or Complete

If more tasks remain:
- Move to next task
- Re-read plan (may have new context)
- Continue cycle

If all tasks complete:
- Final verification pass
- Integration testing
- Update plan status: **COMPLETE**

## Two-Stage Review

After completing any substantial task:

### Stage 1: Spec Compliance
- Does this match what the plan asked for?
- Are acceptance criteria met?
- No scope creep, no missing pieces?

### Stage 2: Code Quality
- Clean, readable code?
- Tests adequate?
- No obvious bugs or issues?
- Follows project conventions?

**If issues found:**
- Fix immediately for small issues
- Update plan and re-implement for significant issues

## Handling Blockers

**If stuck on a task:**

1. **Document the blocker**
   - What were you trying to do?
   - What happened?
   - What did you try?

2. **Update plan with blocker**
   ```markdown
   ### Task 3: [Task Name] ⚠️ BLOCKED
   
   **Blocker:** [Description]
   **Attempted:** [What you tried]
   **Next:** [Proposed resolution or need help]
   ```

3. **Options:**
   - Ask user for guidance
   - Skip and return later
   - Re-plan the approach

## Bug Loop Discipline

**If you discover a bug during execution:**

1. **STOP** - Do not apply quick fix
2. **Document** - Note the bug in plan file
3. **Return to Plan Phase** - Update `tmp/plan-*.md` with:
   - Root cause analysis
   - Correct approach
   - Fix strategy
4. **Review** - Verify updated plan
5. **Implement Fix** - Follow the updated plan
6. **Verify** - Confirm fix works

**Never:** Apply ad-hoc fixes without updating the plan.

## Parallel Task Execution

For independent tasks (advanced):

1. Identify tasks with no dependencies between them
2. Implement each in isolation
3. Full integration test

## Completion Criteria

Plan is **COMPLETE** when:

- [ ] All tasks checked off
- [ ] All tests passing (unit + integration)
- [ ] No TODO comments remaining
- [ ] Code reviewed (if required)
- [ ] Documentation updated (if required)
- [ ] Manual verification complete (if applicable)
- [ ] Constitution compliance verified
- [ ] Plan file updated with final status

## Example Task Execution

**Plan Task:**
```markdown
### Task 2: Read TDP Value from asusd

**Files:**
- Modify: `rust_ui/src/backend/asusd.rs`
- Test: `rust_ui/src/features/tdp.rs` (fake `TdpService`)

- [ ] **Step 1: Write test for load()**
  - Fake service returns sustained 35 W
  - Expect `TdpState::Loaded` with `sustained_w == 35`

- [ ] **Step 2: Run test - expect FAIL**

- [ ] **Step 3: Implement get_tdp_config()**
  - Read `CurrentValue` of `ppt_pl1_spl` over D-Bus
  - Map into `TdpConfig`
  - Return typed result

- [ ] **Step 4: Run test - expect PASS**

```

**Execution:**
1. Write test → FAIL (method doesn't exist)
2. Implement method → PASS
3. Refactor (extract constant 1000)
4. Update plan: check off all steps
5. Next task
