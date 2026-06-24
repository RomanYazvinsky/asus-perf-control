---
description: Create new skills following TDD best practices
---

# Writing Skills Workflow

## Overview

**Writing skills IS Test-Driven Development applied to process documentation.**

You write test cases (pressure scenarios), watch them fail (baseline behavior), write the skill (documentation), watch tests pass (agents comply), and refactor (close loopholes).

**Core principle:** If you didn't watch an agent fail without the skill, you don't know if the skill teaches the right thing.

## What is a Skill?

A **skill** is a reference guide for proven techniques, patterns, or tools. Skills help future agent instances find and apply effective approaches.

**Skills are:** Reusable techniques, patterns, tools, reference guides

**Skills are NOT:** Narratives about how you solved a problem once

## When to Create a Skill

**Create when:**
- Technique wasn't intuitively obvious to you
- You'd reference this again across projects
- Pattern applies broadly (not project-specific)
- Others would benefit

**Don't create for:**
- One-off solutions
- Standard practices well-documented elsewhere
- Project-specific conventions (put in project.md)
- Mechanical constraints (automate with regex/validation instead)

## User Input

```text
$ARGUMENTS
```

If user wants to create a skill, guide them through the TDD process.

## TDD for Skills (RED-GREEN-REFACTOR)

| TDD Concept | Skill Creation |
|-------------|----------------|
| **Test case** | Pressure scenario with agent |
| **Production code** | Skill document (workflow file) |
| **Test fails (RED)** | Agent violates rule without skill |
| **Test passes (GREEN)** | Agent complies with skill present |
| **Refactor** | Close loopholes while maintaining compliance |

## The Process

### Phase 1: RED - Establish Baseline

**Before writing ANY skill documentation:**

1. **Identify the problem**
   - What behavior do you want to prevent/encourage?
   - What mistakes happen repeatedly?

2. **Create pressure scenario**
   - Describe a situation that triggers the bad behavior
   - Run it with current (unskilled) agent
   - Document how the agent fails

3. **Document violations**
   - Note exact rationalizations used
   - Note what the agent did wrong
   - This becomes your test case

**Example:**
```
Problem: Agent applies quick fixes without root cause analysis

Pressure scenario: "The app crashes on startup. Fix it."

Baseline response (fail):
- Agent adds try-catch
- Agent restarts service
- "Should work now" (without verification)
- Root cause never identified

Documented violations:
- Symptom patching instead of diagnosis
- No verification of fix
- No root cause understanding
```

### Phase 2: GREEN - Write Minimal Skill

**Write skill addressing ONLY the documented violations:**

```markdown
---
description: Use when debugging any issue, before proposing fixes
---

# Systematic Debugging

## Overview

**Core principle:** ALWAYS find root cause before attempting fixes.

## The Iron Law

```
NO FIXES WITHOUT ROOT CAUSE INVESTIGATION FIRST
```

## The Four Phases

### Phase 1: Root Cause Investigation
- Read error messages carefully
- Reproduce consistently
- Check recent changes
- Formulate hypothesis

### Phase 2: Root Cause Verification
- Test your hypothesis
- Add diagnostic instrumentation
- Verify failing component

### Phase 3: Fix Design
- Minimal, targeted fix
- One thing only

### Phase 4: Implementation & Verification
- Implement designed fix
- Verify fix works
- Verify no regressions
```

**Key:** Address ONLY the violations you documented. Nothing extra.

### Phase 3: REFACTOR - Close Loopholes

**Re-test with variations:**

1. **Run same scenario with skill**
   - Verify agent now follows process
   - Document success

2. **Create pressure variations**
   - "I'm in a hurry, just fix it"
   - "This is a simple bug"
   - "Users are waiting"

3. **Find new rationalizations**
   - Document how agent tries to bypass
   - Add counter-measures to skill

4. **Iterate**
   - RED: New variation fails
   - GREEN: Update skill
   - Refactor: Improve clarity

## SKILL.md Structure

**Frontmatter (YAML):**

```yaml
---
name: skill-name-with-hyphens
description: Use when [specific triggering conditions and symptoms]
---
```

**Guidelines:**
- `name`: letters, numbers, hyphens only
- `description`: Start with "Use when..." - focus on WHEN to use, not what it does
- Max 1024 characters total for frontmatter

**Body structure:**

```markdown
# Skill Name

## Overview
What is this? Core principle in 1-2 sentences.

## When to Use
- Symptoms that trigger this skill
- Specific situations
- When NOT to use

## Core Pattern / Process
Steps to follow with examples

## Quick Reference
Table or bullets for scanning

## Common Mistakes
What goes wrong + fixes

## Anti-Patterns to Avoid
| Bad | Why | Good |
|-----|-----|------|
```

## For Windsurf: Workflow File Structure

**Location:** `.windsurf/workflows/sp.<skill-name>.md`

**Naming:**
- Use `sp.` prefix for superpowers skills
- Short, descriptive name
- Lowercase with hyphens

**Structure adapted for Windsurf:**

```markdown
---
description: Use when [triggering conditions]
---

# Skill Name Workflow

## Overview
Core principle

## User Input
Capture $ARGUMENTS if needed

## When to Use
Symptoms and situations

## The Process
Step-by-step instructions

## Examples
Concrete examples

## Integration with Project
How this fits your specific project
```

## Testing Methodology

### Test Case Structure

```markdown
## Test: [Scenario Name]

**Setup:** [Context provided to agent]

**Expected behavior with skill:**
- [ ] Agent does X
- [ ] Agent does Y
- [ ] Agent does NOT do Z

**Baseline (without skill):**
- Agent did: [document actual behavior]
- Rationalization: [what agent said]

**With skill:**
- Agent did: [document new behavior]
- Pass/Fail: [result]
```

### Pressure Test Categories

1. **Time pressure:** "Quick fix needed"
2. **Complexity dismissal:** "This is simple"
3. **Authority pressure:** "Boss wants it now"
4. **Experience bias:** "I've seen this before"
5. **Partial solutions:** "This should be enough"

## Common Skill Writing Mistakes

| Mistake | Why It's Wrong | Fix |
|---------|----------------|-----|
| Writing what skill does (not when) | Agent doesn't know when to load it | "Use when..." in description |
| No baseline testing | Skill may not address real problem | RED phase mandatory |
| Too comprehensive | Hard to follow, bloated | Minimal addressing violations |
| No anti-patterns | Agent rationalizes around skill | Document specific loopholes |
| Project-specific content | Not reusable | Extract patterns, remove specifics |
| Narrative format | Hard to scan, unclear steps | Structured, bulleted, tables |

## Example: Writing a Skill

**Problem observed:**
Agent commits code without running tests when "in a hurry"

**Baseline test:**
```
User: "Just commit this fix quickly, tests take too long"

Agent (FAIL):
- "I'll commit now and run tests after"
- Commits untested code
- "Should be fine"
```

**Skill written:**

```markdown
---
description: Use when committing code, especially under time pressure
---

# Verify Before Commit

## Overview

**Core principle:** Unverified code is not ready to commit.

## The Iron Law

```
NO COMMIT WITHOUT VERIFICATION
```

## Verification Checklist

- [ ] Tests pass locally
- [ ] Code compiles/builds
- [ ] Manual check of changed functionality
- [ ] Lint/static analysis clean

## Anti-Patterns

| Rationalization | Response |
|-----------------|----------|
| "I'll test after commit" | Commit = claim it's ready. Verify first. |
| "This change is simple" | Simple changes break things too |
| "CI will catch it" | Local feedback is faster than CI |
| "In a hurry" | Rushing causes rework, which is slower |

## Time Pressure Response

When user says "quickly":
1. Run minimal verification (at least build + affected tests)
2. If verification fails, report immediately
3. Do NOT commit untested code regardless of urgency
```

**Re-test:**
```
User: "Just commit this fix quickly"

Agent (PASS):
- "I need to verify first, even for quick fixes"
- Runs build
- Runs affected tests
- Then commits
```

## Skill Maintenance

**Skills evolve:**

1. **Collect observations**
   - When did agent fail to follow skill?
   - What rationalization was used?

2. **Update skill**
   - Add anti-pattern
   - Clarify instruction
   - Close loophole

3. **Re-verify**
   - Test updated skill
   - Ensure compliance

## Integration with Your Constitution

When writing skills for this project:

- Reference Constitution principles
- Include Constitution compliance checks
- Make principles actionable through process

**Example integration:**
```markdown
## Constitution Compliance Check

- [ ] Principle I (Hardware Safety): Safety validation in place?
- [ ] Principle VI (Single Responsibility): File size limits respected?
- [ ] Principle VII (Plan-Review-Implement): Plan exists before implementation?
```

## Remember

**Skills are tested documentation.**

- RED: Watch agent fail without skill
- GREEN: Write minimal skill to make agent comply
- REFACTOR: Close loopholes with pressure testing

**No skill without testing.**
