ADR-004: Trust Graph Architecture

* Status: Proposed
* Date: 2026-11-07
* Decision: Adopt a dedicated, agent-centric Trust Graph architecture for HoloTrust.

1. Context

HoloTrust is evolving from decentralized identity, verifiable attestations, and contextual reputation into a broader agent-centric trust network.

The Trust Graph represents explicit relationships between agents without relying on a centralized identity provider or a universal trust score.

The architecture must preserve agent autonomy, directional relationship semantics, contextual meaning, privacy, and compatibility with Holochain.

2. Decision

Implement Trust Graph as a dedicated domain with separate integrity and coordinator zomes.

The Trust Graph will build on the existing identity, attestation, and reputation capabilities without merging their responsibilities.

Version 0.1 will represent explicit trust declarations only. It will not introduce evidence fields, numerical trust scores, automatic reciprocal relationships, or transitive trust calculations.

3. Core Principles

1. Agent-centric ownership and control.
2. No centralized trust authority.
3. Directional relationships.
4. Context-dependent trust declarations.
5. Immutable historical records.
6. Privacy by design.
7. Explicit authorization for relationship removal.
8. Backward compatibility with existing modules.
9. Eventual consistency must be reflected honestly in query behavior.

4. Directional Trust Model

Trust relationships are directional.

An agent may express a trust relationship toward another agent without implying that the other agent trusts them in return.

The system must distinguish:

* Trust given.
* Trust received.
* Mutual trust.
* No established relationship.

Mutual trust is determined only when both agents have independently created active relationships toward one another in the same context.

A one-way relationship must never be interpreted as mutual.

5. Contextual Trust

Version 0.1 supports these stable context identifiers:

* general
* commerce
* professional
* hospitality
* technical
* community
* reliability

Unknown contexts must be rejected.

Context identifiers are protocol values and must remain stable. Display labels may be localized.

A relationship in one context must not automatically imply a relationship in another context.

6. Relationship Data Model

The initial conceptual relationship structure is:

pub struct TrustRelationship {
    pub schema_version: u8,
    pub subject_agent: AgentPubKey,
    pub context: TrustContext,
}

The relationship author must be derived from the authenticated Holochain action author.

The author must not be accepted as a user-controlled input field.

The relationship target is represented by subject_agent.

An agent must not create a relationship with itself.

Version 0.1 must not include:

* A numerical trust score.
* A universal trust ranking.
* An automatic reciprocal relationship.
* An automatic transitive trust calculation.
* Sensitive personal information.
* Attestation or reputation evidence fields.

7. Relationship Lifecycle

The system must support:

* Creation.
* Retrieval.
* Contextual querying.
* Removal.
* Historical inspection.

Removing a relationship must create a separate immutable removal record.

Only the original relationship author may remove that relationship.

The original relationship entry must never be edited or deleted.

A removed relationship must not be presented as active.

Creating a new relationship after removal creates a new historical declaration.

8. Module Boundaries

The implementation will use:

* trust_integrity: entry definitions, validation rules, link types, and integrity constraints.
* trust: coordinator functions, relationship lifecycle, and queries.

The Trust Graph will reuse existing Identity capabilities for agent identity and discovery.

Attestation remains responsible for verifiable claims and their historical status.

Reputation remains responsible for contextual evaluation of evidence.

Trust Graph records explicit agent-declared relationships only.

Version 0.1 must not modify Attestation or Reputation records and must not duplicate their data models.

Future evidence-aware integrations may be considered through a separate architectural decision.

9. Query Capabilities

The initial API should support:

* create_trust_relationship
* remove_trust_relationship
* get_trust_relationship
* get_trust_relationships_by_author
* get_trust_relationships_for_subject
* get_trust_relationships_by_context
* get_trust_relationship_history

Queries must distinguish active relationships from removed relationships.

The system must support checking directional and mutual relationships.

Discovery links must be validated by the integrity zome.

Historical records must remain retrievable even if discovery links are removed.

Results reflect records available to the querying agent at query time. The API must not claim globally instantaneous DHT consistency.

10. Security and Privacy

The implementation must:

* Validate all externally supplied input.
* Enforce ownership and authorization rules.
* Prevent unauthorized relationship modification.
* Reject self-directed relationships.
* Reject duplicate active relationships.
* Reject duplicate removal records.
* Avoid exposing private information through public queries.
* Avoid storing sensitive personal information in public DHT entries.
* Avoid treating a trust relationship as proof of legal identity or objective trustworthiness.

11. Frontend Integration

The Trust Graph must eventually provide frontend capabilities for:

* Finding another agent.
* Creating a contextual trust relationship.
* Viewing outgoing relationships.
* Viewing incoming relationships.
* Identifying mutual relationships.
* Removing relationships created by the current agent.
* Inspecting relationship history.

The interface must clearly distinguish incoming and outgoing relationships.

The frontend must use the real Holochain conductor integration rather than permanent mock data.

No universal trust score may be displayed.

12. Testing Requirements

Tests must cover:

* Relationship creation.
* Authenticated author attribution.
* Directional relationship semantics.
* Supported context acceptance.
* Unknown context rejection.
* Self-relationship rejection.
* Duplicate active relationship rejection.
* Authorized removal.
* Unauthorized removal rejection.
* Duplicate removal rejection.
* Historical record preservation.
* Active and removed relationship queries.
* Mutual relationship queries.
* Multi-agent discovery.
* Context-specific query behavior.
* DHT consistency behavior.
* Regression compatibility with Identity, Attestation, and Reputation.

Multi-agent tests should use at least three independent agents.

Static checks and focused tests must be completed before expensive multi-conductor integration tests.

Fast CI must pass before Full CI is started.

13. Implementation Sequence

1. Approve HT-TRUST-001 and this ADR.
2. Inspect existing workspace, DNA manifests, and coordinator patterns.
3. Implement Trust integrity types and validation.
4. Implement relationship and removal record validation.
5. Implement discovery links.
6. Implement Trust coordinator operations.
7. Add focused integrity and coordinator tests.
8. Register Trust zomes in Cargo workspace and DNA manifest.
9. Add multi-agent integration tests.
10. Integrate Trust Graph into the frontend.
11. Run static checks and focused tests.
12. Run Fast CI.
13. Run Full CI only after preliminary checks pass.
14. Update architecture, roadmap, and project continuity documentation.

14. Consequences

Positive consequences:

* A dedicated and extensible Trust Graph domain.
* Clear separation between identity, reputation, attestation, and trust.
* Directional and contextual relationship semantics.
* Immutable and auditable relationship history.
* A foundation for future agent-to-agent and economic interactions.

Trade-offs:

* Additional zomes and validation logic.
* More integration and multi-agent testing.
* Additional frontend and query complexity.

These trade-offs are accepted because the Trust Graph is a core part of the HoloTrust architecture.

15. Acceptance Criteria

This ADR is considered implemented when:

* Trust integrity and coordinator zomes exist.
* Directional and contextual relationship rules are enforced.
* Relationship lifecycle and history behavior are tested.
* Authorization rules are enforced by integrity validation.
* DNA and workspace registration are complete.
* Frontend relationship discovery and management work.
* Required CI workflows pass.
* Documentation accurately reflects the implemented architecture.
