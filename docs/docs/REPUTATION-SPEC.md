HoloTrust Reputation Specification

* Specification: HT-REPUTATION-001
* Version: 0.1.0
* Status: Draft
* Scope: Contextual, agent-centric reputation

1. Purpose

HoloTrust reputation measures an agent’s demonstrated experience within a specific context.

The system MUST NOT create a universal reputation score.

A participant may have different reputation results in different domains.

2. Core Principles

1. Reputation is contextual, not universal.
2. Reputation must be connected to verifiable records.
3. Every review must have an identifiable author.
4. Agents cannot review themselves.
5. An agent may publish only one review per subject and domain.
6. Published reviews must remain historically auditable.
7. Reputation calculations must be transparent.
8. Sensitive personal information must not be stored in public DHT entries.

3. Initial Reputation Domains

The first version defines these domains:

* Commerce — trade and commercial interactions.
* Professional — professional services and work.
* Hospitality — accommodation, tourism and hospitality.
* Technical — technical work and software services.
* Community — community participation and cooperation.
* Reliability — fulfillment of commitments and dependable behavior.

Domain identifiers MUST be stable and validated.

Unknown domains MUST be rejected until explicitly supported.

4. Reputation Review Model

The next review schema must support:

* schema_version
* subject_agent
* domain
* rating
* comment

Requirements:

* Rating MUST be between 1 and 5.
* Comment MUST be optional and limited to 500 characters.
* Author MUST be derived from the authenticated Holochain action.
* Subject and author MUST be different agents.
* Domain MUST be a supported domain identifier.

5. Duplicate Prevention

An agent may publish only one review for the same subject in the same domain.

A review in another domain is a separate evaluation.

Duplicate prevention MUST be enforced at the integrity layer, not only by coordinator functions.

6. Legacy Compatibility

Existing schema version 1 reviews MUST remain readable.

They MUST NOT be silently assigned to a contextual domain.

Legacy reviews are unclassified and MUST be excluded from domain-specific reputation calculations unless a future explicit migration policy is adopted.

New contextual reviews MUST use a versioned schema that does not break existing records.

7. Reputation Queries

The reputation API must support:

* Retrieve reviews for an agent.
* Retrieve reviews for an agent in a specific domain.
* Calculate average rating per domain.
* Return review count per domain.
* Return the records supporting each result.

Empty domains MUST return zero reviews and an average of 0.0.

Results MUST be deterministic.

8. Calculation Rules

For each domain:

average_rating = sum(valid ratings) / valid review count

The API MUST return:

* domain
* review_count
* average_rating

A result MUST NOT be interpreted as a universal measure of an agent’s value or trustworthiness.

9. Security and Integrity

The integrity layer MUST reject:

* Unsupported schema versions.
* Unknown reputation domains.
* Ratings outside 1–5.
* Self-reviews.
* Duplicate reviews by the same author for the same subject and domain.
* Unauthorized review links.
* Review edits or deletions that violate historical auditability.

Coordinator validation alone is insufficient.

10. Frontend Requirements

The interface must allow users to:

* Select a reputation domain.
* Publish a review in that domain.
* View domain-specific review history.
* View average rating and review count per domain.
* Inspect the individual reviews behind a result.

The interface MUST NOT display a single global reputation score.

11. Implementation Sequence

1. Add the versioned contextual review schema.
2. Preserve compatibility with existing reviews.
3. Implement domain validation in the integrity zome.
4. Implement domain-aware duplicate prevention.
5. Implement domain-specific coordinator queries.
6. Add multi-agent integration tests.
7. Update the reputation frontend.
8. Verify the complete CI pipeline.

12. Definition of Done

* [ ]	Existing reviews remain readable.
* [ ]	New contextual reviews can be created.
* [ ]	Unsupported domains are rejected.
* [ ]	Duplicate reviews are rejected per domain.
* [ ]	Different domains can be reviewed independently.
* [ ]	Domain-specific summaries are correct.
* [ ]	Multi-agent tests pass.
* [ ]	Frontend displays contextual reputation.
* [ ]	Full CI pipeline passes.

⸻

HoloTrust principle: Identity establishes the agent. Evidence establishes the claim. Context establishes the meaning of reputation.
