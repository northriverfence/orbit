Message Format
<type>(<scope>): <short imperative summary>

<body – optional, wrapped at 72 cols>
- What changed and why
- Any caveats / migrations

Refs: #123


Types: feat, fix, chore, docs, refactor, perf, test, build, ci.

Good

fix(auth): reject expired refresh tokens and add metrics


Avoid

update stuff


One logical change per commit. Rebase to clean up WIP before merging.
