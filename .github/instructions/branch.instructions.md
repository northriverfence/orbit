Default model: trunk-based with short-lived branches.

Naming: {type}/{short-kebab-summary}
Examples:

feat/auth-guard-admin

fix/order-total-rounding

chore/deps-2025-11

Flow

git checkout -b feat/x

Commit small, atomic changes.

Rebase on latest main before PR: git fetch && git rebase origin/main

Resolve conflicts locally; push with --force-with-lease.
