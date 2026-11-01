Contributing to HoloTrust

Thank you for your interest in contributing to HoloTrust.

HoloTrust is an open-source project exploring decentralized identity, verifiable attestations, contextual reputation, and trust relationships using Holochain.

Our goal is to build secure, privacy-preserving infrastructure for humans, AI agents, and services.

1. Core Principles

All contributions should respect these principles:

* Security before convenience.
* Privacy by design.
* Decentralization and agent-centric architecture.
* Verifiable claims instead of unsupported assertions.
* Contextual reputation instead of simplistic global scores.
* Small, testable, documented changes.
* Open collaboration and technical transparency.

2. Development Workflow

1. Review the project vision and architecture.
2. Check the roadmap to understand the current development phase.
3. Open an issue before proposing major architectural changes.
4. Keep changes focused and independently testable.
5. Include tests for new functionality.
6. Update documentation when behavior or architecture changes.
7. Submit changes through a pull request.

3. Code Quality

Contributions should:

* Follow established Rust conventions.
* Use meaningful names.
* Avoid unnecessary dependencies.
* Handle errors explicitly.
* Keep integrity validation separate from coordinator logic.
* Include comments where design decisions are not obvious.
* Avoid introducing undocumented behavior.

Formatting and linting requirements will be finalized after the Holochain toolchain is verified.

4. Security and Privacy

Security-sensitive changes require particular care.

Contributors must:

* Never publish private keys, secrets, or credentials.
* Avoid storing sensitive personal information in public DHT entries.
* Validate all externally supplied entry data.
* Verify author and ownership constraints.
* Consider replay, impersonation, and malicious input scenarios.
* Never treat possession of a public key as proof of civil identity.
* Document security assumptions and limitations.

Report suspected vulnerabilities privately to the project maintainers rather than publishing exploit details in an issue.

5. Testing

New functionality should include appropriate tests.

Depending on the feature, tests may cover:

* Valid and invalid inputs.
* Integrity validation.
* Authorization and ownership.
* Multi-agent interactions.
* Link creation and retrieval.
* Update and revocation behavior.
* Adversarial or unexpected inputs.

No feature should be considered complete solely because it compiles.

6. Architectural Changes

Major changes to identity, attestations, reputation, or trust relationships should include an Architecture Decision Record (ADR).

An ADR should explain:

* The problem.
* The proposed decision.
* Alternatives considered.
* Security and privacy implications.
* Expected consequences.

7. Pull Request Checklist

Before submitting a pull request:

* [ ]	The change has a clear purpose.
* [ ]	Related tests are included.
* [ ]	Documentation is updated.
* [ ]	No secrets or sensitive data are included.
* [ ]	Security implications have been considered.
* [ ]	The change is consistent with the project architecture.

8. Current Development Status

HoloTrust is in its foundation stage.

The initial priority is to establish a verified Holochain development environment and implement the Identity MVP.

Contributors should avoid introducing premature economic integrations or reputation scoring before the identity and attestation foundations are stable.

9. Project Direction

HoloTrust aims to contribute to an agent-centric internet where identity, trust, and reputation can be independently verified without depending on a single central authority.

Every contribution should help move the project toward that goal.

⸻

HoloTrust — Building the trust infrastructure of the agent-centric internet.
