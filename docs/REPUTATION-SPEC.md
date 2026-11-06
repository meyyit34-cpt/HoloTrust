# HoloTrust Reputation Specification

- Specification: HT-REPUTATION-001
- Version: 0.1.0
- Status: Draft
- Scope: Contextual, agent-centric reputation
- Related decision: ADR-003

## 1. Purpose

HoloTrust reputation measures an agent's demonstrated experience within a specific context.

The system MUST NOT create a universal reputation score.

A participant may have different reputation results in different domains.

## 2. Core Principles

1. Reputation is contextual, not universal.
2. Reputation must be connected to verifiable records.
3. Every review must have an identifiable author.
4. Agents cannot review themselves.
5. An agent may publish only one review per subject and domain.
6. Published reviews must remain historically auditable.
7. Reputation calculations must be transparent.
8. Sensitive personal information must not be stored in public DHT entries.
9. Reputation evaluations do not independently prove that a claim is true.
10. Results must expose their provenance and supporting records.

## 3. Initial Reputation Domains

Version 0.1 defines these stable identifiers:

- commerce: Trade and commercial interactions
- professional: Professional services and work
- hospitality: Accommodation, tourism and hospitality
- technical: Technical work and software services
- community: Community participation and cooperation
- reliability: Fulfillment of commitments and dependable behavior

Identifiers are protocol values and MUST remain stable.

Display labels may be localized.

Unknown domains MUST be rejected until explicitly supported by a protocol update.

## 4. Contextual Review Model

The new contextual review schema must support:

- schema_version
- subject_agent
- domain
- rating
- comment
- evidence_attestations

Requirements:

- Schema version MUST equal the supported contextual review version.
- Rating MUST be between 1 and 5.
- Comment MUST be optional and limited to 500 characters.
- Author MUST be derived from the authenticated Holochain action.
- Subject and author MUST be different agents.
- Domain MUST be one of the stable identifiers above.
- At least one bounded evidence attestation reference MUST be supplied.
- Every evidence reference MUST resolve to a valid attestation concerning the reviewed subject.

## 5. Duplicate Prevention and Immutability

An agent may publish only one contextual review for the same subject in the same domain.

A review in another domain is a separate evaluation.

Duplicate prevention MUST be enforced at the integrity layer, not only by coordinator functions or DHT queries.

Published contextual reviews MUST NOT be edited or deleted.

Historical records remain available.

## 6. Legacy Compatibility

Existing schema-version-1 ReputationReview entries MUST remain readable through the existing Identity module.

They MUST NOT be silently assigned to a contextual domain.

Legacy reviews are unclassified and MUST be excluded from domain-specific reputation calculations unless a future explicit, auditable migration policy is adopted.

New contextual reviews MUST use a separate versioned entry type in the Reputation module.

Existing Identity zomes and legacy APIs remain operational.

## 7. Evidence and Provenance

The Reputation module consumes Attestation records; it does not issue or modify attestations.

A successful attestation signature or hash check proves origin and integrity, not the truth of the underlying claim.

The system MUST distinguish:

- Attestation record validity.
- Issuer attribution.
- Subject binding.
- Attestation status: active, revoked, expired, or superseded.
- Truth of the underlying claim.

A missing, invalid, revoked, expired, or superseded attestation MUST NOT be silently represented as active supporting evidence.

## 8. Reputation Queries

The reputation API must support:

- Retrieve contextual reviews for an agent.
- Retrieve contextual reviews for an agent in a specific domain.
- Calculate average rating per domain.
- Return review count per domain.
- Return the records supporting each result.
- Return evidence references and provenance.

Empty domains MUST return zero reviews and an average of 0.0.

Results MUST be deterministic for the records available to the querying agent at query time.

Holochain DHT eventual consistency MUST be documented.

## 9. Calculation Rules

For each domain:

average_rating = sum(valid ratings) / valid review count

The API MUST return:

- domain
- review_count
- average_rating
- supporting review records
- evidence provenance

A result MUST NOT be interpreted as a universal measure of an agent's value or trustworthiness.

## 10. Security and Integrity

The integrity layer MUST reject:

- Unsupported schema versions.
- Unknown reputation domains.
- Ratings outside 1–5.
- Self-reviews.
- Duplicate reviews by the same author for the same subject and domain.
- Missing or invalid evidence references.
- Evidence attestations concerning a different subject.
- Unauthorized review links.
- Review edits or deletions that violate historical auditability.

Coordinator validation alone is insufficient.

## 11. Frontend Requirements

The interface must allow users to:

- Select a reputation domain.
- Publish a review in that domain with supporting attestation references.
- View domain-specific review history.
- View average rating and review count per domain.
- Inspect individual reviews and evidence behind a result.

The interface MUST NOT display a single global reputation score.

## 12. Implementation Sequence

1. Add the versioned contextual review schema in a separate Reputation integrity zome.
2. Preserve compatibility with existing Identity reviews.
3. Implement domain and evidence validation in the integrity zome.
4. Implement domain-aware duplicate prevention.
5. Implement domain-specific coordinator queries.
6. Add multi-agent integration tests.
7. Update the reputation frontend to use the new zome.
8. Verify the complete CI pipeline and real-conductor smoke test.

## 13. Definition of Done

- [ ] Existing legacy reviews remain readable.
- [ ] Legacy reviews are excluded from contextual summaries.
- [ ] New contextual reviews can be created.
- [ ] Unsupported domains are rejected.
- [ ] Duplicate reviews are rejected per domain.
- [ ] Different domains can be reviewed independently.
- [ ] Evidence attestations are validated and status-aware.
- [ ] Domain-specific summaries are correct.
- [ ] Provenance is returned and inspectable.
- [ ] Multi-agent tests pass.
- [ ] Frontend displays contextual reputation without a global score.
- [ ] Full CI pipeline passes.

---

HoloTrust principle:

Identity establishes the agent.
Attestation records the claim.
Evidence supports verification.
Context establishes the meaning of reputation.
