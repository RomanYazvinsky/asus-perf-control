---
description: Finishing a development branch - merge/PR decision workflow
---

# Finishing a Development Branch Workflow

## Overview

**Purpose:** Decide when and how to merge a completed feature branch.

"Done" with implementation doesn't mean "ready to merge." This workflow guides the decision process for finishing a branch cleanly.

## When to Use

- Feature implementation is complete
- All plan tasks checked off
- Tests passing
- Ready to integrate to main

## Pre-Merge Checklist

### 1. Completion Verification

- [ ] All plan tasks complete (`tmp/plan-*.md`)
- [ ] All tests passing (unit + integration)
- [ ] No TODO/FIXME comments remaining
- [ ] Code follows project conventions
- [ ] Constitution compliance verified

### 2. Constitution Compliance

**Principle I - Hardware Safety:**
- [ ] Safety validation in place
- [ ] No unconfirmed hardware operations

**Principle VI - Single Responsibility:**
- [ ] Files under line limits (Dart <150, Rust <200)
- [ ] No god objects created

**Principle VII - Plan-Review-Implement:**
- [ ] Plan file complete and updated
- [ ] Bug fixes documented in plan

### 3. Review Status

- [ ] Self-review completed
- [ ] Peer review (if required) approved
- [ ] All feedback addressed
- [ ] No outstanding safety issues

### 4. Documentation

- [ ] Commit messages clear and descriptive
- [ ] Complex logic documented
- [ ] User-facing changes documented (if applicable)
- [ ] Breaking changes noted

## Merge vs PR Decision

### Direct Merge (Fast-Forward)

**When appropriate:**
- Solo project
- Small, safe changes
- No review required
- Linear history preferred

```bash
# Ensure clean state
git checkout main
git pull origin main
git merge --ff-only feature-branch

# If not fast-forward capable:
git rebase main feature-branch
git checkout main
git merge --ff-only feature-branch
```

### Pull Request (Merge Commit or Rebase)

**When appropriate:**
- Team project
- Significant changes
- Review required
- Want discussion record

```bash
# Push branch
git push -u origin feature-branch

# Create PR via GitHub CLI or web
git pr create --title "Feature: X" --body "Implements..."
```

## The Decision Matrix

| Factor | Direct Merge | PR Required |
|--------|--------------|-------------|
| Team size | Solo | 2+ people |
| Change size | <10 lines | >10 lines |
| Risk level | Low | Medium/High |
| Review policy | Not required | Required |
| Safety critical | No | Yes (hardware control) |
| History | Linear preferred | Discussion valued |

## Merge Process

### 1. Sync with Main

```bash
git checkout main
git pull origin main
git checkout feature-branch
git rebase main  # or: git merge main
```

**Resolve any conflicts.**

### 2. Final Verification

```bash
# Run full test suite
flutter test
# or
cargo test
# or
make test
```

### 3. Prepare Commit History (Optional)

```bash
# Clean up messy commits
git rebase -i main

# Actions:
# - squash: combine small commits
# - reword: fix commit messages
# - drop: remove debug commits
```

### 4. Execute Merge

**Fast-forward:**
```bash
git checkout main
git merge --ff-only feature-branch
```

**Merge commit:**
```bash
git checkout main
git merge --no-ff feature-branch -m "Merge: Feature description"
```

**Squash merge:**
```bash
git checkout main
git merge --squash feature-branch
git commit -m "Feature: Description"
```

### 5. Push and Clean Up

```bash
# Push main
git push origin main

# Delete local branch
git branch -d feature-branch

# Delete remote branch
git push origin --delete feature-branch
```

## Post-Merge Verification

- [ ] CI passes on main
- [ ] Integration tests pass
- [ ] Manual smoke test (if applicable)
- [ ] Monitoring/alerts (if applicable)

## Rollback Plan

**If merge causes issues:**

```bash
# Immediate rollback (if not pushed)
git reset --hard HEAD~1

# If already pushed - revert
git revert -m 1 <merge-commit-hash>
git push origin main
```

**Document rollback:**
- What failed
- Why it failed
- Fix required before re-attempt

## Integration with Plan

**Update plan file:**
```markdown
## Status: ✅ COMPLETE - MERGED

**Merged:** 2026-04-10
**Merge commit:** abc1234
**PR:** #123 (if applicable)

**Verification:**
- [x] CI passed
- [x] Integration tests passed
- [x] Manual testing complete
```

## User Input

```text
$ARGUMENTS
```

If user indicates they want to finish a branch, guide them through the checklist and decision process.

## Common Scenarios

### "I'm done but not sure if it's ready"

→ Run through Pre-Merge Checklist. If any unchecked, not ready.

### "Do I need a PR?"

→ Check Decision Matrix. When in doubt, create PR.

### "Main has changed since I started"

→ Rebase or merge main into your branch, resolve conflicts, re-test.

### "Tests pass locally but I'm worried"

→ That's what CI is for. Push and see. If CI passes, confidence increases.
