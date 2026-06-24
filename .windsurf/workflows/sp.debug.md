---
description: Systematic debugging - 4-phase root cause analysis before any fixes
---

# Systematic Debugging Workflow

## Overview

**Core principle:** ALWAYS find root cause before attempting fixes. Symptom fixes are failure.

```
NO FIXES WITHOUT ROOT CAUSE INVESTIGATION FIRST
```

## When to Use

- Test failures
- Bugs in production  
- Unexpected behavior
- Performance problems
- Build failures
- Integration issues

**Use ESPECIALLY when:**
- Under time pressure (emergencies make guessing tempting)
- "Just one quick fix" seems obvious
- You've already tried multiple fixes
- Previous fix didn't work

## The Four Phases

You MUST complete each phase before proceeding to the next.

### Phase 1: Root Cause Investigation

**BEFORE attempting ANY fix:**

1. **Read Error Messages Carefully**
   - Don't skip past errors or warnings
   - They often contain the exact solution
   - Read stack traces completely
   - Note line numbers, file paths, error codes

2. **Reproduce Consistently**
   - Can you trigger it reliably?
   - What are the exact steps?
   - Does it happen every time?
   - If not reproducible → gather more data, don't guess

3. **Check Recent Changes**
   - What changed that could cause this?
   - Git diff, recent commits
   - New dependencies, config changes
   - Environmental differences

4. **Gather Evidence in Multi-Component Systems**
   - Log what data enters/exits each component
   - Verify environment/config propagation
   - Check state at each layer
   - Run once to gather evidence showing WHERE it breaks
   - THEN analyze evidence to identify failing component

5. **Formulate Root Cause Hypothesis**
   - State clearly what you think is causing the bug
   - Include your reasoning
   - Identify the specific component
   - Be specific enough to test

**Phase 1 Output:** Written hypothesis of root cause with supporting evidence.

**DO NOT PROCEED TO PHASE 2 without this output.**

### Phase 2: Root Cause Verification

**Test your hypothesis.**

Design an experiment that would prove/disprove your hypothesis:

```
- What specific observation would confirm my hypothesis?
- What observation would disprove it?
- What's the simplest way to test this?
```

**Verification methods:**
- Add targeted logging
- Create minimal reproduction case
- Use debugger to inspect state
- Add temporary assertions
- Check system state directly

**Result:**
- ✓ Hypothesis confirmed → Proceed to Phase 3
- ✗ Hypothesis disproven → Return to Phase 1 with new hypothesis

### Phase 3: Fix Design

**ONLY after confirmed root cause.**

Design minimal, targeted fix:

```
- What exactly needs to change?
- Where is the minimal change point?
- How do we verify this fix works?
- What could this fix break?
```

**Constraints:**
- Fix ONE thing
- Minimal surface area
- No "while I'm here" changes
- No refactoring

### Phase 4: Implementation & Verification

1. **Implement the designed fix**
2. **Verify fix works**
3. **Verify no regressions**
4. **Document root cause** in commit message

**Verification Before Completion**

- Run ALL tests, not just the one that was failing
- Check related functionality
- Verify performance hasn't degraded
- Test edge cases

## Anti-Patterns to Avoid

| Anti-Pattern | Why It's Wrong |
|--------------|----------------|
| "Try this quick fix" | Symptoms return, often worse |
| "Google says..." | Your context differs from theirs |
| "Works on my machine" | Environment differences are clues |
| "Just restart it" | Masks the root cause |
| Commenting out code | Hides bugs, doesn't fix them |
| Adding defensive checks | Symptoms without cure |

## Defense in Depth

For critical bugs, add layers of protection:

```
If cause was: Missing validation
- Add validation at input point (primary fix)
- Add validation at boundary
- Log validation failures
- Alert on pattern of failures
```

## User Input

```text
$ARGUMENTS
```

If user provided specific bug/issue, start Phase 1 investigation on that issue.
