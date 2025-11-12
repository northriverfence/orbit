Cadence

Patch: as needed

Minor: when new features land behind stable flags

Major: breaking changes only (rare; require RFC)

Steps

Ensure main is green and version bumped.

Tag:

./scripts/release bump <patch|minor|major>
./scripts/release tag


Changelog: update CHANGELOG.md under the new version with:

Added / Changed / Fixed / Removed

Cut artifacts & publish (CI job).

Smoke tests on staging with checklist.

Promote to prod with a canary (X%) for 30–60 minutes.

Post a release note in #eng-releases.

Rollback

Revert tag: git revert -m 1 <merge_commit>

Disable new flags.

If data change: run ./scripts/migrate --down <version>
