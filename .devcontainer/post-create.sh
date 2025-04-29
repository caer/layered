#!/bin/sh
set -e

# Check if repo is a git lfs repo.
if git lfs ls-files > /dev/null 2>&1; then
    git lfs pull
fi
