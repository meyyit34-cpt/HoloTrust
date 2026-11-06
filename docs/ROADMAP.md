
# HoloTrust — Development Roadmap

## Project Objective

Build a decentralized identity and reputation network using Holochain.

The project evolves through small, testable, security-focused development stages.

---

## Phase 0 — Foundation

Goal: Establish a reliable and reproducible development environment.

Tasks:

- [x] Create GitHub repository.
- [x] Document project vision.
- [x] Document technical architecture.
- [x] Confirm compatible Holochain toolchain in CI.
- [x] Pin Rust toolchain version in CI.
- [x] Pin Holonix to a specific commit.
- [x] Generate canonical Holochain DNA scaffold.
- [x] Configure automated Rust formatting checks.
- [x] Configure and verify linting checks.
- [x] Establish automated CI pipeline.

Status: Complete.

## Phase 1 — Identity MVP

Goal: Implement the first functional identity layer.

Tasks:

- [x] Define identity entry schema.
- [x] Implement identity integrity zome.
- [x] Implement identity coordinator zome.
- [x] Create agent profile.
- [x] Retrieve own profile.
- [x] Retrieve another agent's profile.
- [x] Validate profile ownership.
- [x] Implement profile input validation.
- [x] Add automated tests.
- [x] Verify multi-agent profile operations.

Exit criteria: Met.

Status: Complete.

## Phase 2 — Identity Discovery

Goal: Enable reliable profile discovery and updates.

Tasks:

- [x] Define agent-based profile discovery anchor.
- [x] Implement profile discovery links.
- [x] Define profile update semantics.
- [x] Ensure updates resolve to the latest profile after DHT consistency.
- [x] Test multi-agent profile discovery.
- [x] Test unauthorized and stale profile updates.
- [x] Implement duplicate profile-link rejection.
- [x] Add duplicate profile-link comparison tests.
- [x] Document discovery and update behavior.
- [x] Document DHT consistency and ordering limitations.

Exit criteria:

- [x] Agents can discover published profiles.
- [x] Profile updates resolve to the latest published version after consistency.
- [x] Stale updates are rejected.
- [x] Exact duplicate profile links are rejected.
- [x] Historical profile actions remain available.
- [x] Multi-agent discovery tests pass.

Status: Complete.

Note: Holochain DHT propagation is eventually consistent. A query can only resolve records and links available to that agent at query time. This phase does not claim globally instantaneous ordering across disconnected network states.

## Phase 3 — Verifiable Attestations

Goal: Enable signed, verifiable claims.

Tasks:

- [x] Define attestation schema.
- [x] Define issuer and subject relationships.
- [x] Implement attestation integrity rules.
- [x] Implement attestation coordinator.
- [x] Add issuer verification.
- [x] Define revocation and supersession semantics.
- [x] Verify supersession integrity with dedicated tests.
- [x] Add evidence-reference validation.
- [x] Add multi-agent integration tests.

Exit criteria:

- [x] Claims have verifiable issuers.
- [x] Invalid claims are rejected.
- [x] Historical changes remain auditable.
- [x] Evidence references can be validated.
- [x] Supersession and revocation authorization are tested.
- [x] Multi-agent discovery and verification are tested.

Status: Complete.

Completion notes:

- Dedicated supersession and revocation integration tests are implemented.
- Tests use multiple independent agents and verify third-party discovery.
- Unauthorized supersession and revocation attempts are rejected.
- Historical attestations remain discoverable and auditable.
- Superseded and revoked states are recognized by other agents.
- Full CI run #197 passed build, test, packaging and conductor smoke-test stages.

## Phase 4 — Contextual Reputation

Goal: Build evidence-based, domain-specific reputation.

Tasks:

- [ ] Define stable reputation domains.
- [ ] Define evidence evaluation rules.
- [ ] Design separate reputation integrity and coordinator zomes.
- [ ] Preserve compatibility with existing legacy reviews.
- [ ] Implement domain-specific reputation queries.
- [ ] Prevent universal-score assumptions.
- [ ] Add explainability and provenance.
- [ ] Add multi-agent integration tests.
- [ ] Integrate contextual reputation APIs into the web interface.
- [ ] Verify end-to-end reputation workflows.

Current foundations:

- ADR-003 reputation architecture decision documented.
- Contextual reputation specification drafted.
- Reputation integrity and coordinator modules introduced.
- Domain-based review and evidence-validation logic implemented.

Status: In progress.

## Phase 5 — Trust Graph

Goal: Establish relationships between agents.

Tasks:

- [ ] Define trust relationship schema.
- [ ] Implement relationship creation.
- [ ] Implement relationship removal.
- [ ] Implement relationship queries.
- [ ] Add context-aware trust relationships.
- [ ] Add multi-agent tests.

Status: Not started.

## Phase 6 — Economic Integration

Goal: Explore decentralized economic coordination.

Potential integrations:

- hREA
- Unyt
- Holochain economic infrastructure
- HoloFuel-compatible coordination models

Status: Not started.

---

## Engineering Principles

1. Security before convenience.
2. Small, independently testable changes.
3. Reproducible builds.
4. No sensitive personal data in public DHT entries.
5. Explicit schema and version management.
6. Backward compatibility for published records.
7. Documentation updated with implementation.
8. Existing working code must not be changed without a verified reason.

---

## Current Status

Current phase: Phase 4 — Contextual Reputation.

Completed foundations:

- Holochain DNA and hApp build.
- Identity integrity and coordinator zomes.
- Agent profile creation and retrieval.
- Profile ownership and update validation.
- Multi-agent identity tests.
- Duplicate profile-link protection.
- Identity discovery documentation.
- Automated CI pipeline.
- Rust formatting and Clippy checks verified.
- Attestation schema and integrity validation.
- Attestation coordinator and issuer verification.
- Attestation revocation and supersession logic.
- Dedicated multi-agent supersession and revocation tests.
- Evidence-reference validation.
- Full CI build, test, packaging and conductor smoke-test verification.

Phase 4 foundations:

- Reputation architecture decision documented.
- Contextual reputation specification drafted.
- Separate reputation integrity and coordinator modules introduced.
- Domain-specific review and evidence-validation logic introduced.

Next milestone:

Complete contextual reputation multi-agent integration tests and verify domain-specific reputation workflows.

Following milestone:

Integrate contextual reputation APIs into the web interface and complete Phase 4 end-to-end verification.

---

HoloTrust — Building the trust infrastructure of the agent-centric internet.
