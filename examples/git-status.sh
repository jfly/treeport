#!/usr/bin/env bash

set -euo pipefail

# Check for a dirty git tree (untracked, unstaged, or staged changed).
if [ -n "$(git status --porcelain)" ]; then
    echo "dirty"
# Check for unpushed branches.
elif [ -n "$(git log --branches --not --remotes)" ]; then
    echo "unpushed"
else
    echo "synced"
fi
