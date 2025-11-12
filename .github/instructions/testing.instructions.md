Pyramid

Unit tests: fast, deterministic, no network.

Integration: real boundaries (DB, queue), hermetic fixtures.

E2E: critical happy paths only.

Commands
make test
make test.unit
make test.integration
make test.e2e

Writing tests

Name: mirrors module under tests/…

Arrange-Act-Assert; avoid hidden work in fixtures.

Use data builders; avoid random sleeps.

Coverage

Target: 80% lines, 100% in core packages.

PRs must not decrease coverage on touched files.
