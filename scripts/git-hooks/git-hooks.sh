#!/usr/bin/env bash
chmod 755 scripts/git-hooks/commit-msg scripts/git-hooks/pre-commit
cp scripts/git-hooks/commit-msg scripts/git-hooks/pre-commit .git/hooks/
