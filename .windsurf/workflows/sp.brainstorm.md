---
description: Socratic design refinement - clarify and improve specs
---

# Brainstorming Workflow

## Overview

**Purpose:** Refine specifications through Socratic questioning.

Before writing code or even making a plan, make sure we're building the right thing. This skill uses structured questioning to surface unstated assumptions, identify gaps, and clarify scope.

## When to Use

- New feature requests
- Vague requirements
- "We should add X" statements
- Scope conflicts
- Design disagreements
- Before writing implementation plans

## User Input

```text
$ARGUMENTS
```

Take user's feature idea/requirements and begin Socratic questioning.

## The Process

### 1. Clarify the Problem

What problem are we solving?

**Ask:**
- What specific pain point does this address?
- Who experiences this problem?
- How are they solving it now?
- What makes the current solution inadequate?

**Goal:** Ensure we're solving a real problem, not implementing a solution looking for a problem.

### 2. Define Success

How will we know this works?

**Ask:**
- What does "done" look like?
- How will users interact with this?
- What specific behaviors should change?
- What metrics (if any) indicate success?

**Goal:** Concrete acceptance criteria, not vague "it should work better."

### 3. Explore Scope

What's in and what's out?

**Ask:**
- What's the minimum viable version?
- What related features might we be tempted to add?
- What are we explicitly NOT doing?
- Are there different modes/use cases?
- What platforms/contexts matter?

**Goal:** Clear boundaries. Prevent scope creep before it starts.

### 4. Identify Dependencies & Constraints

What does this touch?

**Ask:**
- What existing systems does this interact with?
- What data does it need? Where does that come from?
- What permissions/authentication is required?
- Are there performance requirements?
- What are the hardware safety implications? (Constitution Principle I)

**Goal:** Surface hidden complexity early.

### 5. Challenge Assumptions

What are we taking for granted?

**Ask:**
- Why must it work this specific way?
- What if we approached it differently?
- Are there simpler alternatives?
- What would we do if we couldn't use our preferred approach?
- What technical debt are we accepting?

**Goal:** Avoid false constraints and premature optimization.

### 6. Risk Check

What could go wrong?

**Ask:**
- What could break existing functionality?
- What safety considerations apply? (hardware, data, security)
- What's the rollback strategy if this fails?
- What happens at scale? At minimum usage?

**Goal:** Identify risks before they become problems.

## Output Format

After questioning, produce:

```markdown
# Refined Specification: [Feature Name]

## Problem Statement
[Clear description of the problem being solved]

## Success Criteria
- [ ] Specific, testable criterion 1
- [ ] Specific, testable criterion 2

## Scope

### In Scope
- [ ] Item 1
- [ ] Item 2

### Out of Scope (future considerations)
- [ ] Item 3 (v2)
- [ ] Item 4 (future)

## Key Decisions

| Decision | Rationale |
|----------|-----------|
| Choice A | Why we chose this |
| Choice B | Alternative rejected because... |

## Open Questions
- [ ] Question 1 (need input from...)
- [ ] Question 2 (technical research needed)

## Dependencies
- Dependency 1
- Dependency 2

## Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Risk A | Medium | High | Strategy |
| Risk B | Low | Medium | Strategy |

## Constitution Compliance

- [ ] Principle I (Hardware Safety): [assessment]
- [ ] Principle VI (Single Responsibility): [assessment]
- [ ] Principle VII (Plan-Review-Implement): Will create plan at `tmp/plan-*.md`
```

## Next Steps

After refining the spec:
1. Save to `tmp/spec-<feature>.md`
2. Get user sign-off
3. Use `/sp.plan` to create implementation plan
4. Use `/sp.execute` to implement

## Red Flags to Surface

**Watch for and challenge:**
- "Just add a button that..." (what does the button do?)
- "Like [competitor], but..." (do we understand why they did it?)
- "Users want..." (which users? how do we know?)
- "It should be simple..." (famous last words)
- "We can just..." (just is a lie)

## Example Session

**User:** "Add a feature to control fan curves."

**Questions:**
- What specific scenarios require manual fan control vs automatic?
- What safety limits must we enforce? (thermal damage risk)
- How do users interact with this - GUI, CLI, both?
- What happens if user sets dangerous values?
- Do we validate against hardware limits? (Constitution I - Hardware Safety)
- Which laptops/models support this?

**Refined:** Manual fan curve control with thermal safety limits, per-model validation, GUI only for v1, CLI for v2.
