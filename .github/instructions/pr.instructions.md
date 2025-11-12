Goal

Open small, reviewable PRs that ship safely and fast.

Checklist (before you open)

PR is ≤ ~300 lines changed (prefer smaller); split if bigger.

Title is imperative and scoped: feat(router): add auth guard for /admin

Description explains the why, not just the what.

Includes a brief test plan and screenshots/CLI output if relevant.

Links the issue/ticket: Closes #123.

Migration/rollback steps documented if needed.

No secrets or flaky sleeps; feature-flags behind risky changes.

PR Template
### Why
<problem or user story — 1–3 sentences>

### What
- <bullet point summary of changes>

### How (approach)
- <key design choices, tradeoffs, alternatives>

### Test Plan
- [ ] Unit: <cmd/output>
- [ ] Integration: <cmd/output>
- [ ] Manual: <steps + result>
- [ ] Screenshots (if UI): <img/links>

### Risks & Rollback
- Risk: <blast radius, failure mode>
- Rollback: <git revert / flag off / data restore steps>

### Links
Closes #<issue>; Design doc: <url> (if any)

Review Expectations

Author: request 2 reviewers; mark “Ready for review” only when green.

Reviewers: aim <24h>, comment on correctness, boundaries, readability.

Nits with nit:; blocking with blocker:.
