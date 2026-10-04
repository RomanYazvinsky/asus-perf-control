---
description: Request and respond to code review with structured checklists
---

# Code Review Workflow

## Overview

**Purpose:** Structured code review - both requesting and receiving.

Code review is not about catching mistakes. It's about:
- Sharing knowledge
- Maintaining standards
- Catching assumptions
- Improving design

## Part 1: Requesting Review

### When to Request

- Before merging any non-trivial change
- When touching critical/safety code (hardware control)
- When unsure about design decisions
- After completing a plan implementation

### Review Request Checklist

Before requesting review, verify:

```markdown
## Pre-Review Checklist

### Functionality
- [ ] Code works as specified
- [ ] Tests pass (all of them)
- [ ] Edge cases handled
- [ ] Error paths tested

### Code Quality
- [ ] Clear, readable code
- [ ] Good naming (functions, variables, files)
- [ ] No dead code or commented-out sections
- [ ] Appropriate abstractions

### Safety (Constitution Principle I)
- [ ] Hardware safety validation present
- [ ] Input bounds checking
- [ ] Safe defaults
- [ ] No dangerous operations without confirmation

### Architecture (Constitution Principle VI)
- [ ] Single Responsibility: each file/class has one job
- [ ] File size: Rust <200 lines
- [ ] No god objects
- [ ] Clear boundaries

### Testing
- [ ] Tests exist for new functionality
- [ ] Tests fail appropriately when code is broken
- [ ] Edge cases covered
- [ ] Integration points tested

### Documentation
- [ ] Complex logic explained
- [ ] Public APIs documented
- [ ] Commit messages clear
- [ ] Plan file updated (if applicable)
```

### Preparing Review Context

**Provide reviewer with:**

```markdown
## Review Request

**What:** [Brief description of change]

**Why:** [Problem being solved / Feature being added]

**How:** [High-level approach]

**Key Files:**
- `rust_ui/src/features/feature.rs` - [what it does]
- `rust_ui/src/features/feature/tests.rs` - [test coverage]

**Plan Reference:** `tmp/plan-feature.md` (if applicable)

**Specific Concerns:**
- [ ] Is the error handling adequate?
- [ ] Does the abstraction make sense?
- [ ] Any performance concerns?

**Testing Done:**
- Unit tests: [coverage]
- Integration: [what was tested]
- Manual: [if applicable]
```

### Review Request Process

1. **Self-review first** - Go through your own code with fresh eyes
2. **Run automated checks** - Tests, lint, type check
3. **Prepare context** - Write review request document
4. **Submit** - Create PR/ask for review with context

## Part 2: Receiving Review

### Mindset

- Reviews are about the code, not you
- Reviewers are trying to help
- "I don't understand" is valid feedback
- Better to catch issues now than in production

### Responding to Feedback

**For each comment:**

1. **Acknowledge** - "Good point" / "I see what you mean"
2. **Decide** - Accept, discuss, or decline (with reason)
3. **Act** - Make change or explain why not
4. **Resolve** - Mark resolved when done

### Types of Feedback

**Clarification Questions:**
- Answer directly
- If code is unclear, improve the code

**Style/Nitpicks:**
- Generally accept
- Not worth fighting over

**Design Concerns:**
- Take seriously
- Discuss if you disagree
- Consider broader implications

**Bug Reports:**
- Thank the reviewer
- Write test that reproduces
- Fix and verify

**Safety Issues:**
- **ALWAYS address**
- No exceptions
- Constitution Principle I

### Handling Disagreement

**If you disagree with feedback:**

1. **Assume good intent** - Reviewer sees something you don't
2. **Ask for clarification** - "Can you say more about...?"
3. **Explain your reasoning** - "I did X because Y"
4. **Seek consensus** - "What do you think about Z instead?"
5. **Escalate if needed** - Get third opinion for significant disagreements

### Declining Feedback (Rare)

**Valid to decline when:**
- Suggestion is out of scope
- Creates significant technical debt
- Violates project constraints

**How to decline:**
```
"Good suggestion, but out of scope for this change. 
Captured in issue #123 for follow-up."
```

## Review Integration with Plan

**If review finds issues:**

1. **Document in plan** - `tmp/plan-*.md`
2. **Create task** - Add "Address review feedback" task
3. **Prioritize** - Critical issues first
4. **Re-implement** - Following bug loop discipline
5. **Re-request review** - When addressed

## Review Completion

**Review is complete when:**

- [ ] All feedback addressed or declined (with reason)
- [ ] Reviewer approves
- [ ] No unresolved safety issues
- [ ] Plan updated (if issues found)
- [ ] Ready to merge

## Quick Reference

| Situation | Action |
|-----------|--------|
| Reviewer found bug | Thank, write test, fix, verify |
| Style feedback | Accept and fix |
| Design concern | Discuss, consider trade-offs |
| Safety issue | **Fix immediately, no debate** |
| "I don't understand" | Clarify code, add comments |
| Out of scope suggestion | Decline politely, capture for later |
