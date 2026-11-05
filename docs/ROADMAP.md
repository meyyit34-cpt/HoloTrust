
# HoloTrust — Development Roadmap

## Project Objective

Build a decentralized identity and reputation network using Holochain.

The project will evolve through small, testable, security-focused development stages.

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
- [ ] Configure and verify linting checks.
- [x] Establish automated CI pipeline.

Exit criteria:

- Reproducible CI toolchain references.
- Successful clean build.
- DNA and hApp packaging.
- Workspace and integration tests pass.

Status: Complete, with linting configuration remaining.

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

Exit criteria:

- Profiles can be created and retrieved.
- Invalid entries are rejected.
- Unauthorized profile updates are rejected.
- Multi-agent tests pass.
- CI build and integration tests pass.

Status: Complete.

## Phase 2 — Identity Discovery

Goal: Enable reliable profile discovery and updates.

Tasks:

- [x] Define agent-based profile discovery anchor.
- [x] Implement profile discovery links.
- [x] Define profile update semantics.
- [x] Ensure updates resolve to the latest profile.
- [x] Test multi-agent profile discovery.
- [x] Test unauthorized and stale profile updates.
- [ ] Add explicit duplicate profile-link handling tests.
- [ ] Document discovery and update behavior.

Exit criteria:

- Agents can discover published profiles.
- Profile updates resolve deterministically.
- Stale updates are rejected.
- Duplicate or conflicting links are handled safely.
- Multi-agent discovery tests pass.

Status: In progress.

## Phase 3 — Verifiable Attestations

Goal: Enable signed, verifiable claims.

Tasks:

- [ ] Define attestation schema.
- [ ] Define issuer and subject relationships.
- [ ] Implement attestation integrity rules.
- [ ] Implement attestation coordinator.
- [ ] Add issuer verification.
- [ ] Define revocation and supersession semantics.
- [ ] Test evidence references.
- [ ] Add multi-agent integration tests.

Exit criteria:

- Claims have verifiable issuers.
- Invalid claims are rejected.
- Historical changes remain auditable.
- Evidence references can be verified.

Status: Not started.

## Phase 4 — Contextual Reputation

Goal: Build evidence-based reputation.

Tasks:

- [ ] Define stable reputation domains.
- [ ] Define evidence evaluation rules.
- [ ] Design separate reputation integrity and coordinator zomes.
- [ ] Preserve compatibility with existing legacy reviews.
- [ ] Implement domain-specific reputation queries.
- [ ] Prevent universal-score assumptions.
- [ ] Add explainability and provenance.
- [ ] Add multi-agent integration tests.

Exit criteria:

- Reputation is contextual.
- Results can be traced to supporting evidence.
- Legacy reviews remain readable.
- No universal reputation score is produced.

Status: Not started.

## Phase 5 — Trust Graph

Goal: Establish relationships between agents.

Tasks:

- [ ] Define trust relationship schema.
- [ ] Implement relationship creation.
- [ ] Implement relationship removal.
- [ ] Implement relationship queries.
- [ ] Add context-aware trust relationships.
- [ ] Add multi-agent tests.

Exit criteria:

- Trust relationships are verifiable and queryable.
- Relationship history and ownership are validated.

Status: Not started.

## Phase 6 — Economic Integration

Goal: Explore decentralized economic coordination.

Potential integrations:

- hREA
- Unyt
- Holochain economic infrastructure
- HoloFuel-compatible coordination models

This phase begins only after identity, attestations, and contextual reputation foundations are stable.

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

Current phase: Phase 2 — Identity Discovery.

Completed foundations:

- Holochain DNA and hApp build.
- Identity integrity and coordinator zomes.
- Agent profile creation and retrieval.
- Profile ownership and update validation.
- Multi-agent identity tests.
- Automated CI pipeline.
- Pinned Rust and Holonix CI references.

Next milestone:

Complete duplicate profile-link handling tests and discovery documentation.

Following milestone:

Begin Phase 3 — Verifiable Attestations.

---

HoloTrust — Building the trust infrastructure of the agent-centric internet.
