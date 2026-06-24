---
description: Parallel development branches using git worktrees
---

# Git Worktrees Workflow

## Overview

**Purpose:** Work on multiple branches simultaneously without stashing or switching.

Git worktrees allow you to have multiple branches checked out at the same time in separate directories. No more "stash my changes, switch branch, do work, switch back, unstash."

## When to Use

- Reviewing a PR while working on your own feature
- Long-running test on one branch, need to fix a bug on another
- Comparing behavior between branches
- Parallel development of independent features
- Running CI-like checks without disturbing your working tree

## Commands Reference

### Create Worktree

```bash
# Create new worktree for existing branch
git worktree add ../<branch-name>-wt <branch-name>

# Create worktree with new branch
git worktree add -b feature-x ../feature-x-wt main
```

### List Worktrees

```bash
git worktree list
```

### Remove Worktree

```bash
# Remove worktree (after cd-ing to main repo)
git worktree remove ../<branch-name>-wt

# Force remove if dirty
git worktree remove -f ../<branch-name>-wt
```

### Prune Stale Worktrees

```bash
git worktree prune
```

## Workflow: Review While Developing

**Scenario:** You're working on `feature-a`, need to review `feature-b` PR.

```bash
# From main repo (feature-a branch)
git worktree add ../feature-b-wt feature-b

# In another terminal
cd ../feature-b-wt

# Review, test, make changes
git diff
git commit -am "Review feedback"

# Push from worktree
git push

# Back in main repo - feature-b still untouched
cd ../<main-repo>

# Clean up when done
git worktree remove ../feature-b-wt
```

## Workflow: Parallel Feature Development

**Scenario:** Two independent features, want to work on both without switching.

```bash
# From main on main repo
git worktree add -b feature-tdp ../tdp-wt
git worktree add -b feature-fan ../fan-wt

# Terminal 1: TDP work
cd ../tdp-wt
# ... implement TDP feature

# Terminal 2: Fan work  
cd ../fan-wt
# ... implement Fan feature

# Both can be tested independently
# Both can be pushed independently
```

## Workflow: Long Test + Bug Fix

**Scenario:** Integration tests running on `feature-x`, bug reported on `main`.

```bash
# Tests are running in ../feature-x-wt (don't touch!)

# Create emergency fix worktree
git worktree add -b hotfix-urgent ../hotfix-wt main

cd ../hotfix-wt
# ... fix bug ...
git commit -am "Fix critical bug"
git push

# Tests still running undisturbed in feature-x-wt
```

## Best Practices

### Naming Convention

```
../<branch-name>-wt    # Simple suffix
../wt-<branch-name>    # Prefix
../wt/<branch-name>     # Subdirectory (cleaner)
```

### Directory Structure

```
~/projects/
  myproject/           # Main repo (main branch)
  myproject-wt/
    feature-a/         # Worktree for feature-a
    feature-b/         # Worktree for feature-b
    hotfix/            # Worktree for hotfix
```

### IDE Integration

**VS Code:**
- Open each worktree as separate window
- Or add to workspace: File → Add Folder to Workspace

**IntelliJ/Windsurf:**
- Open worktree directories as separate projects
- Or attach directories to existing project

### Avoiding Conflicts

**Don't:**
- Run build processes that write to shared directories
- Modify git config that affects all worktrees
- Have worktrees nested inside each other

**Do:**
- Use separate build directories per worktree
- Configure IDE per-worktree settings
- Keep worktrees at same directory level as main repo

## Clean Up

**Remove specific worktree:**
```bash
git worktree remove ../branch-wt
```

**Remove all worktrees (dangerous):**
```bash
git worktree list | grep -v "(bare)" | awk '{print $1}' | xargs git worktree remove
```

## Common Issues

### "fatal: '<path>' is already registered"

```bash
git worktree prune
git worktree add ../new-wt branch
```

### "fatal: could not create leading directories"

```bash
# Parent directory doesn't exist
mkdir -p ../worktrees
git worktree add ../worktrees/branch-wt branch
```

### Branch already checked out in another worktree

```bash
# Can't have same branch in multiple worktrees
git worktree list  # Find where branch is
git worktree remove <that-worktree>  # Or use different branch
```

## User Input

```text
$ARGUMENTS
```

If user specifies a worktree operation (create, list, remove), execute it. Otherwise, explain worktree usage.

## Integration with Superpowers

**During /sp.execute:**
- Can create worktrees for parallel tasks
- Each task in its own worktree
- Merge back when complete

**During /sp.review:**
- Create worktree for PR review
- Doesn't disturb current work
- Clean up after merge

**Example:**
```bash
# Review a colleague's PR
git worktree add ../pr-123-wt pr-branch
cd ../pr-123-wt

# Full review environment
# Run tests, check code, make suggestions

git worktree remove ../pr-123-wt
```
