# GitHub Security Settings

GitHub CodeQL default setup is required for the public repository.

This project intentionally does not add an advanced CodeQL workflow because
GitHub rejects duplicate SARIF uploads when default setup and an advanced
workflow both analyze the same repository.

Before a release tag:

1. Open repository settings.
2. Go to Code security.
3. Confirm CodeQL analysis default setup is active for the default branch.
4. Confirm the latest analysis completed successfully for the release commit.
5. Record that check in the pentest report scope or notes.
