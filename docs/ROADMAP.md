HoloTrust — Development Roadmap

Project Objective

Build a decentralized, agent-centric identity, attestation, reputation, and trust network using Holochain.

Development proceeds through independently testable, security-focused phases.

⸻

Phase 0 — Foundation

Goal: Establish a reliable and reproducible development environment.

* [x]	Create GitHub repository.
* [x]	Document project vision.
* [x]	Document technical architecture.
* [x]	Confirm compatible Holochain toolchain in CI.
* [x]	Pin Rust toolchain version in CI.
* [x]	Pin Holonix to a specific commit.
* [x]	Generate canonical Holochain DNA scaffold.
* [x]	Configure automated Rust formatting checks.
* [x]	Configure and verify linting checks.
* [x]	Establish automated CI pipeline.

Status: Complete.

Phase 1 — Identity MVP

Goal: Implement the first functional identity layer.

* [x]	Define identity entry schema.
* [x]	Implement identity integrity zome.
* [x]	Implement identity coordinator zome.
* [x]	Create agent profile.
* [x]	Retrieve own profile.
* [x]	Retrieve another agent’s profile.
* [x]	Validate profile ownership.
* [x]	Implement profile input validation.
* [x]	Add automated tests.
* [x]	Verify multi-agent profile operations.

Exit criteria: Met.

Status: Complete.

Phase 2 — Identity Discovery

Goal: Enable reliable profile discovery and updates.

* [x]	Define agent-based profile discovery anchor.
* [x]	Implement profile discovery links.
* [x]	Define profile update semantics.
* [x]	Ensure updates resolve to the latest profile after DHT consistency.
* [x]	Test multi-agent profile discovery.
* [x]	Test unauthorized and stale profile updates.
* [x]	Implement duplicate profile-link rejection.
* [x]	Add duplicate profile-link comparison tests.
* [x]	Document discovery and update behavior.
* [x]	Document DHT consistency and ordering limitations.

Exit criteria: Met.

Status: Complete.

Note: Holochain DHT propagation is eventually consistent. Queries reflect records and links available to the querying agent at query time.

Phase 3 — Verifiable Attestations

Goal: Enable signed, verifiable claims.

* [x]	Define attestation schema.
* [x]	Define issuer and subject relationships.
* [x]	Implement attestation integrity rules.
* [x]	Implement attestation coordinator.
* [x]	Add issuer verification.
* [x]	Define revocation and supersession semantics.
* [x]	Verify supersession integrity with dedicated tests.
* [x]	Add evidence-reference validation.
* [x]	Add multi-agent integration tests.

Exit criteria: Met.

Status: Complete.

Completion notes:

* Dedicated supersession and revocation integration tests are implemented.
* Multi-agent tests verify third-party discovery.
* Unauthorized supersession and revocation attempts are rejected.
* Historical attestations remain discoverable and auditable.
* Full CI run #197 passed its build, test, packaging, and conductor smoke-test stages.

Phase 4 — Contextual Reputation

Goal: Build evidence-based, domain-specific reputation.

* [x]	Define stable reputation domains.
* [x]	Define evidence evaluation rules.
* [x]	Design separate reputation integrity and coordinator zomes.
* [x]	Preserve compatibility with existing legacy reviews.
* [x]	Implement domain-specific reputation queries.
* [x]	Prevent universal-score assumptions.
* [x]	Add explainability and provenance.
* [x]	Add multi-agent integration tests.
* [x]	Integrate contextual reputation APIs into the web interface.
* [x]	Verify end-to-end reputation workflows.
* [x]	Document reputation architecture in ADR-003.
* [x]	Finalize reputation specification.

Exit criteria: Met.

Status: Complete.

Completion notes:

* Contextual reputation uses a separate integrity and coordinator zome pair.
* Domain-specific reviews and evidence validation are implemented.
* Legacy reviews remain readable and excluded from contextual summaries.
* Historical evidence status is handled through Attestation integration.
* Multi-agent reputation and legacy compatibility tests passed.
* Fast CI passed.
* Full CI run #42371037589 completed successfully.
* All required build, formatting, packaging, conductor smoke-test, and workspace test jobs passed.
* Reputation specification status was updated to Implemented.

Phase 5 — Trust Graph

Goal: Establish explicit, directional, context-aware relationships between agents.

Design and architecture

* [x]	Create initial Trust Graph specification HT-TRUST-001.
* [ ]	Review and approve Trust Graph specification.
* [ ]	Document architectural decision ADR-004.
* [ ]	Confirm module boundaries and security rules.
* [ ]	Finalize relationship and removal data models.

Integrity and coordinator implementation

* [ ]	Create trust_integrity zome.
* [ ]	Create trust coordinator zome.
* [ ]	Register both zomes in Cargo workspace.
* [ ]	Register both zomes in DNA manifest.
* [ ]	Implement directional relationship creation.
* [ ]	Implement context validation.
* [ ]	Reject self-relationships.
* [ ]	Reject duplicate active relationships.
* [ ]	Implement immutable relationship removal records.
* [ ]	Enforce removal authorization.
* [ ]	Implement relationship discovery and queries.
* [ ]	Preserve historical relationship records.

Testing and integration

* [ ]	Add integrity validation tests.
* [ ]	Add authorization and duplicate tests.
* [ ]	Add three-agent discovery tests.
* [ ]	Add context-specific query tests.
* [ ]	Verify DHT consistency behavior.
* [ ]	Integrate Trust Graph APIs into the frontend.
* [ ]	Display incoming and outgoing relationships separately.
* [ ]	Verify Fast CI.
* [ ]	Verify Full CI and real-conductor behavior.

Status: Design in progress.

Important architectural constraints:

* Relationships are directional.
* Reciprocal trust is never automatic.
* Trust is context-specific.
* Only the original author can remove a relationship.
* Historical records remain auditable.
* No universal trust score.
* No automatic transitive trust calculation.
* No sensitive personal information in public DHT records.
* Trust Graph does not modify Attestation or Reputation records.

Phase 6 — Economic Integration

Goal: Explore decentralized economic coordination.

Potential integrations:

* hREA
* Unyt
* Holochain economic infrastructure
* HoloFuel-compatible coordination models

Status: Not started.

⸻

Engineering Principles

1. Security before convenience.
2. Small, independently testable changes.
3. Reproducible builds.
4. No sensitive personal data in public DHT entries.
5. Explicit schema and version management.
6. Backward compatibility for published records.
7. Documentation updated with implementation.
8. Existing working code must not be changed without a verified reason.
9. Static analysis before expensive integration tests.
10. Fast CI and targeted tests before Full CI.
11. Full CI only after preliminary checks pass.

⸻

Current Status

Completed phases:

* Phase 0 — Foundation.
* Phase 1 — Identity MVP.
* Phase 2 — Identity Discovery.
* Phase 3 — Verifiable Attestations.
* Phase 4 — Contextual Reputation.

Current phase:

Phase 5 — Trust Graph.

Current milestone:

Finalize and approve HT-TRUST-001 and ADR-004 before implementation.

Next implementation sequence:

1. Approve Trust Graph specification.
2. Accept ADR-004.
3. Inspect existing workspace and DNA configuration.
4. Design trust integrity data structures and validation.
5. Implement trust_integrity.
6. Implement trust coordinator.
7. Add targeted tests before expensive multi-agent tests.
8. Verify Fast CI.
9. Run Full CI only after all preliminary checks pass.
10. Integrate the frontend.

⸻

HoloTrust — Building the trust infrastructure of the agent-centric internet.

Identity establishes the agent.

Attestation records the claim.

Evidence supports verification.

Reputation evaluates evidence in context.

Trust Graph records declared relationships between agents.
