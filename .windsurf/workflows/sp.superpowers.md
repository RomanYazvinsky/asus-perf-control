---
description: Introduction to the superpowers skills system and available workflows
---

# Using Superpowers Workflow

## Overview

**Superpowers** is an agentic skills framework that provides structured workflows for software development.

Unlike general instructions, these workflows:
- Trigger automatically based on context
- Provide specific, actionable guidance
- Enforce best practices through process
- Adapt to your project's needs

## Available Workflows in This Project

### Core Development Workflows

| Workflow | Command | What It Does | When to Use |
|----------|---------|--------------|-------------|
| Brainstorm | `/sp.brainstorm` | Socratic questioning to refine ideas | Starting a new feature, vague requirements, scope discussions |
| Plan | `/sp.plan` | Write comprehensive implementation plans | Before coding, breaking down specs into tasks |
| TDD | `/sp.tdd` | RED-GREEN-REFACTOR cycle guidance | Writing tests first, any code change |
| Execute | `/sp.execute` | Task-by-task plan implementation | Following a written plan, subagent-driven development |
| Debug | `/sp.debug` | 4-phase root cause analysis | Any bug, test failure, unexpected behavior |
| Review | `/sp.review` | Code review workflows | Preparing for review, responding to feedback |
| Verify | `/sp.verify` | Verification before completion | Before marking done, before merge |
| Git Worktree | `/sp.git-worktree` | Parallel branch development | Reviewing PRs while coding, multiple features |
| Finish | `/sp.finish` | Merge/PR decision workflow | Feature complete, ready to integrate |
| Writing Skills | `/sp.writing-skills` | Create new skills with TDD | Adding new workflows, improving processes |

### Project-Specific Workflows (Specify Framework)

| Workflow | Command | What It Does | When to Use |
|----------|---------|--------------|-------------|
| Specify | `/speckit.specify` | Create feature specifications | New features, requirements gathering |
| Plan | `/speckit.plan` | Generate implementation plans | After spec is ready, before coding |
| Tasks | `/speckit.tasks` | Create task breakdowns | Converting plans to actionable items |
| Clarify | `/speckit.clarify` | Identify underspecified areas | Requirements unclear, missing details |
| Checklist | `/speckit.checklist` | Generate feature checklists | Quality assurance, verification |
| Implement | `/speckit.implement` | Execute implementation tasks | Ready to code, plan is approved |
| Analyze | `/speckit.analyze` | Cross-artifact consistency analysis | Reviewing specs/plans/tasks alignment |

## How to Use

### 1. Direct Invocation

Type the workflow command to activate it:

```
/sp.brainstorm help me design a new fan control feature
```

The workflow will:
- Acknowledge activation
- Guide you through its process
- Ask clarifying questions
- Produce structured output

### 2. Automatic Triggering

Many workflows trigger automatically based on context:

| Context | Auto-triggered Workflow |
|---------|------------------------|
| "I have a bug..." | `sp.debug` |
| "Let's plan this feature..." | `sp.brainstorm` → `sp.plan` |
| "Help me implement..." | `sp.plan` → `sp.execute` |
| "How should I test..." | `sp.tdd` |
| "Review this code..." | `sp.review` |

### 3. Workflow Chaining

Workflows naturally chain together:

```
Brainstorm (refine idea)
    ↓
Plan (create implementation plan)
    ↓
Execute (implement task-by-task)
    ↓
Verify (confirm it works)
    ↓
Review (code review)
    ↓
Finish (merge)
```

## Workflow Selection Guide

**Starting a new feature:**
1. `/sp.brainstorm` - Refine the idea
2. `/sp.plan` - Create implementation plan
3. `/sp.execute` - Implement
4. `/sp.verify` - Verify it works
5. `/sp.finish` - Merge

**Debugging:**
1. `/sp.debug` - Follow systematic process
2. `/sp.tdd` - Write test for the fix
3. `/sp.verify` - Verify the fix

**Code review:**
1. `/sp.review` - Prepare or respond to review

**Parallel work:**
1. `/sp.git-worktree` - Set up parallel branches
2. `/sp.execute` - Work on each independently

## Integration with Constitution

Superpowers workflows align with your project Constitution:

**Principle I (Hardware Safety):**
- `/sp.plan` requires safety consideration
- `/sp.tdd` enforces test-before-implementation
- `/sp.verify` requires safety verification

**Principle VI (Single Responsibility):**
- `/sp.plan` enforces file size limits
- `/sp.execute` checks for god objects
- `/sp.review` includes SRP checklist

**Principle VII (Plan-Review-Implement):**
- `/sp.plan` creates `tmp/plan-*.md`
- `/sp.execute` follows the plan strictly
- `/sp.debug` returns to plan on bugs

## Getting Help

**During any workflow:**
- Ask questions about the process
- Request clarification on steps
- Challenge requirements that don't fit

**Workflow not working as expected?**
- Check that input matches expected format
- Verify all prerequisites met
- Ask for help with specific step

## Best Practices

1. **Start with the right workflow** - Don't use `/sp.execute` without a plan
2. **Complete each phase** - Don't skip steps in `/sp.debug`
3. **Trust the process** - Workflows exist to prevent common failures
4. **Adapt when needed** - Workflows are guidance, not prison
5. **Give feedback** - If workflow doesn't fit, say so

## Workflow Philosophy

**Why structured workflows?**

> "The essence of programming is not writing code, but thinking clearly about problems. These workflows provide thinking structures."

**Key principles:**
- **Process over intuition** - Following a good process beats ad-hoc guessing
- **Explicit over implicit** - Write down the plan, don't keep it in your head
- **Verification over assumption** - Prove it works, don't assume it does
- **Safety over speed** - Correct takes longer than quick, but rework takes longest

## Quick Reference

```
Need to...                    Use...
-----------------------------|------------------
Refine an idea                /sp.brainstorm
Plan a feature                /sp.plan
Implement a plan              /sp.execute
Debug an issue                /sp.debug
Write tests first             /sp.tdd
Request code review           /sp.review
Verify it works               /sp.verify
Work on multiple branches     /sp.git-worktree
Finish and merge              /sp.finish

Everything about a feature    /speckit.specify → /speckit.plan → /speckit.implement
```

## Remember

**Superpowers don't replace thinking - they structure it.**

You still make decisions. You still apply judgment. But now you have a framework that catches common mistakes and enforces proven practices.

**Your codebase + Superpowers + Constitution = High-quality, maintainable software.**
