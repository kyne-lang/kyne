# Security Policy

Kyne's security governance is fully specified in
[`docs/GOVERNANCE.md` §11](../docs/GOVERNANCE.md#11-security-governance).
This file is a short, discoverable pointer to it — `GOVERNANCE.md` remains
the sole normative source.

## Reporting a vulnerability

**Do not open a public GitHub issue for a suspected vulnerability.**

Report it privately using GitHub's private vulnerability reporting
feature (Security → Report a vulnerability on this repository), so a fix
can be prepared under embargo before the issue becomes public knowledge,
per `GOVERNANCE.md` §11's "Responsible disclosure" and "Security
embargoes" provisions.

## What happens next

1. The report is triaged privately by the Core Maintainers.
2. A fix is prepared under embargo, restricted to those actively working
   on it.
3. A public security advisory is published once the fix is ready,
   describing the issue, its severity, the affected versions, and the fix.
4. A retroactive KIP documenting the fix is filed, per `GOVERNANCE.md`
   §11's "Emergency patches" provision — a security fix MAY ship before
   its KIP completes, but the KIP is still required afterward.

## Supported versions

Per [`GOVERNANCE.md` §9](../docs/GOVERNANCE.md#9-release-policy), the
current major version and the immediately preceding major version both
receive security patches. As of this bootstrap milestone, no version of
Kyne has been released yet — see
[`ROADMAP.md` §10](../docs/ROADMAP.md#10-release-roadmap) for the planned
release sequence.
