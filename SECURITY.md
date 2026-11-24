# Security Policy

## Supported versions

Only the latest version receives fixes. Update to the newest version first.

## Reporting a vulnerability

Do not open a public issue for a security problem. Report it privately through GitHub Security Advisories:

<https://github.com/lucasbrito-wdt/disk-steward/security/advisories/new>

Include the affected version, your macOS version, steps to reproduce, and the impact you see. This is a volunteer
project: expect an acknowledgement within a few days and a fix or a decision as soon as practical. Please allow time for
a fix before disclosing publicly; you will be credited in the release notes unless you prefer otherwise.

## Scope

In scope: the app, the `steward` CLI, the install scripts, and logic that decides which files get compressed (for
example a way to make Disk Steward enable a blocked target or touch files outside the enabled patterns). Out of scope:
vulnerabilities in the `auto-compress-apfs` engine itself, which should go to its own repository, and issues in macOS
or APFS.
