# ADR-003: Contextual Reputation Architecture

- Status: Accepted
- Date: 2026-11-06
- Decision: Separate Contextual Reputation module
- Related specification: HT-REPUTATION-001
- Related decisions: ADR-001, ADR-002

## 1. Context

HoloTrust currently stores schema-version-1 ReputationReview entries in the Identity integrity zome.

These records and their existing coordinator APIs are part of the published compatibility surface.

The existing review model has no domain field. It cannot safely represent contextual reputation, and legacy reviews must not be silently assigned to a domain.

The Attestation module provides attributable, immutable claims that can be evaluated by a separate Reputation module.

## 2. Decision

Contextual reputation MUST be implemented as a separate Holochain zome pair:

- reputation_integrity
- reputation

Existing Identity zomes MUST remain responsible for identity and legacy schema-version-1 reviews.

They MUST NOT become the implementation location for new contextual reputation behavior.

The new module MUST NOT modify, rewrite, or reinterpret existing legacy review records.

## 3. Stable Reputation Domains

Version 0.1 defines these stable identifiers:

- commerce
- professional
- hospitality
- technical
- community
- reliability

These identifiers are protocol values, not display labels.

Unknown identifiers MUST be rejected.

Display labels may be localized without changing the identifiers.

## 4. Contextual Review Records

New contextual reviews MUST use a separately versioned entry type and MUST include:

- schema version
- subject agent
- stable reputation domain
- rating from 1 through 5
- optional comment, maximum 500 characters
- one or more bounded references to supporting attestation records

The author MUST be derived from the authenticated Holochain action.

The author and subject MUST differ.

The integrity zome MUST validate the schema, domain, rating, comment, evidence-reference structure, and author/subject relationship.

## 5. Evidence and Provenance

A contextual review is an evaluation, not proof that its underlying claim is true.

Evidence references MUST resolve to valid, retrievable attestation records concerning the reviewed subject.

The implementation MUST distinguish record validity and issuer attribution from the truth of the underlying claim.

Reputation query results MUST expose supporting review records and evidence references so a client can inspect provenance.

A missing, invalid, revoked, expired, or superseded attestation MUST NOT silently be presented as active supporting evidence.

Status must be resolved using the Attestation module's immutable history rules.

## 6. Duplicate and Historical Rules

An agent may publish at most one contextual review for a given subject and domain.

A review of the same subject in a different domain is a separate evaluation.

Duplicate prevention MUST be enforced by the integrity zome using source-chain history, not only by coordinator checks or DHT link queries.

Published contextual reviews MUST be immutable.

They MUST NOT be updated or deleted.

Corrections require a new, explicitly designed historical mechanism.

No mutable global score is permitted.

## 7. Queries and Calculations

The coordinator MUST support:

- retrieving contextual reviews for a subject
- retrieving reviews for a subject in one domain
- calculating a summary for one domain
- returning review count and average rating
- returning supporting records and provenance

Each summary MUST be scoped to exactly one domain.

Empty domains return zero reviews and an average of 0.0.

Results MUST be deterministic for the records visible to the querying agent at query time.

Holochain DHT eventual consistency MUST be documented.

The API MUST NOT claim globally instantaneous completeness.

No universal reputation score may be calculated or displayed.

## 8. Compatibility

The existing schema-version-1 ReputationReview entry type, its validation behavior, and legacy coordinator externs MUST remain readable and operational.

Legacy reviews MUST be classified as unscoped.

They MUST be excluded from contextual domain summaries unless a future explicit, auditable migration policy is approved.

The new zome pair MUST be added to the DNA without removing or renaming the Identity or Attestation zomes.

## 9. Testing Requirements

Tests MUST cover:

- all supported domain identifiers and unknown-domain rejection
- rating and comment boundaries
- self-review rejection
- author attribution
- duplicate rejection for the same subject and domain
- acceptance of a different domain for the same subject
- evidence existence, subject binding, and attestation status
- immutable review history
- domain-specific summaries and empty-domain behavior
- legacy review readability and exclusion from contextual summaries
- multi-agent DHT discovery and retrieval

## 10. Implementation Order

1. Finalize HT-REPUTATION-001 and this decision.
2. Add the new reputation integrity and coordinator crates.
3. Register both zomes in the DNA manifest and workspace.
4. Implement the versioned contextual review schema and integrity validation.
5. Implement coordinator creation, discovery, and domain-specific queries.
6. Add unit, integrity, and multi-agent tests.
7. Update the frontend to require a domain and show provenance.
8. Run the complete CI pipeline and real-conductor smoke test.

Each step MUST preserve the currently passing Identity and Attestation tests.

## 11. Consequences

Positive:

- Identity and legacy reviews remain backward compatible.
- Contextual reputation has independent consensus rules.
- Reputation results remain domain-specific and explainable.
- Attestation provenance can be inspected instead of hidden behind a score.

Trade-offs:

- A separate zome pair increases implementation and testing work.
- Evidence status resolution requires explicit integration with Attestation.
- DHT results remain eventually consistent.

## 12. Final Decision

HoloTrust adopts the following separation:

Identity establishes the agent.
Attestation records attributable claims.
Evidence supports verification.
Contextual Reputation evaluates evidence within a defined domain.

A global reputation score is explicitly out of scope.
