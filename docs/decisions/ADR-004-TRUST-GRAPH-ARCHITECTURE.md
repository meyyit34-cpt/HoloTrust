ADR-004: Trust Graph Architecture

* Status: Proposed
* Date: 2026-11-07
* Decision: Adopt a dedicated, agent-centric Trust Graph architecture for HoloTrust.

1. Context

HoloTrust is evolving from decentralized identity, verifiable attestations, and contextual reputation into a broader agent-centric trust network.

The Trust Graph must represent relationships between agents without relying on a centralized identity provider or a single global reputation score.

The architecture must preserve user autonomy, verifiable evidence, contextual meaning, and compatibility with Holochain’s agent-centric model.

2. Decision

Implement Trust Graph as a dedicated domain with separate integrity and coordinator zomes.

The Trust Graph will build on existing identity, attestation, and reputation capabilities without merging their responsibilities.

3. Core Principles

1. Agent-centric ownership and control.
2. No centralized trust authority.
3. Context-dependent trust relationships.
4. Verifiable evidence wherever applicable.
5. Immutable historical records.
6. Explicit consent and transparent relationship rules.
7. Privacy by design.
8. Backward compatibility with existing HoloTrust modules.

4. Directional Trust Model

Trust relationships are directional.

An agent may express trust toward another agent without implying that the other agent trusts them in return.

The system must distinguish:

* Trust given.
* Trust received.
* Mutual trust.
* No established relationship.

Mutual trust must never be inferred from a one-way relationship.

5. Contextual Trust

Trust must be associated with a defined context.

Initial context categories:

* Commerce
* Professional
* Hospitality
* Technical
* Community
* Reliability

The architecture must allow additional contexts without requiring a breaking schema migration.

A trust relationship in one context must not automatically imply trust in another context.

6. Relationship Lifecycle

Trust relationships must support:

* Creation.
* Retrieval.
* Contextual querying.
* Removal or revocation.
* Historical inspection.

Existing relationship records must not be silently overwritten.

Removal must preserve the historical record and create an auditable lifecycle event.

7. Evidence and Validation

Where a trust relationship references evidence:

* The referenced evidence must exist.
* The evidence must belong to the relevant subject.
* The evidence must be of an allowed type.
* Invalid or unrelated references must be rejected.
* Evidence status must be resolved through the existing attestation capabilities.

Historical records must remain inspectable even if referenced evidence later expires, is revoked, or is superseded.

The architecture must distinguish historical validity from current evidence status.

8. Module Boundaries

The implementation will use:

* trust_integrity: entry definitions, validation rules, links, and integrity constraints.
* trust: coordinator functions, relationship lifecycle, queries, and cross-zome integration.

The Trust Graph must reuse existing Identity, Attestation, and Reputation capabilities through defined interfaces.

It must not duplicate their core data models unnecessarily.

9. Query Capabilities

The initial API should support:

* Get relationships created by an agent.
* Get relationships received by an agent.
* Query relationships by context.
* Check whether a directional relationship exists.
* Identify mutual relationships.
* Inspect relationship history.
* Retrieve relationship evidence references.

Queries must respect agent authorization and privacy boundaries.

10. Security and Privacy

The implementation must:

* Validate all externally supplied input.
* Enforce ownership and authorization rules.
* Prevent unauthorized relationship modification.
* Avoid exposing private information through public queries.
* Reject malformed or excessive evidence references.
* Avoid treating trust relationships as proof of legal identity.
* Avoid presenting contextual trust as a universal guarantee.

11. Frontend Integration

The Trust Graph must eventually provide frontend capabilities for:

* Viewing an agent’s trust relationships.
* Exploring contextual relationships.
* Inspecting mutual relationships.
* Viewing evidence references.
* Understanding relationship history.
* Managing relationships under the applicable authorization rules.

Frontend implementation must use the real Holochain conductor integration rather than permanent mock data.

12. Testing Requirements

Tests must cover:

* Relationship creation.
* Directional relationship semantics.
* Context validation.
* Invalid input rejection.
* Unauthorized modification rejection.
* Relationship removal and history preservation.
* Evidence reference validation.
* Expired, revoked, and superseded evidence.
* Mutual relationship queries.
* Multi-agent interactions.
* Regression compatibility with Identity, Attestation, and Reputation.

Static checks and focused tests must be completed before expensive multi-conductor integration tests.

Fast CI must pass before Full CI is started.

13. Implementation Sequence

1. Approve this ADR and the Trust Specification.
2. Inspect existing workspace, DNA manifests, and coordinator patterns.
3. Implement Trust integrity types and validation.
4. Implement Trust coordinator operations.
5. Add focused unit and integration tests.
6. Register Trust zomes in the DNA and workspace.
7. Integrate Trust Graph with the frontend.
8. Run static checks and focused tests.
9. Run Fast CI.
10. Run Full CI after all preliminary checks pass.
11. Update architecture, documentation, and project continuity records.

14. Consequences

Positive consequences:

* A dedicated and extensible Trust Graph domain.
* Clear separation between identity, reputation, and trust.
* Directional and contextual relationship semantics.
* Better auditability and evidence integration.
* A foundation for future economic and agent-to-agent interactions.

Trade-offs:

* Additional zomes and validation logic.
* More integration and multi-agent testing.
* Additional frontend and query complexity.

These trade-offs are accepted because the Trust Graph is a core part of the HoloTrust architecture.

15. Acceptance Criteria

This ADR is considered implemented when:

* Trust integrity and coordinator zomes exist.
* Directional and contextual relationship rules are enforced.
* Lifecycle and history behavior are tested.
* Evidence integration works with existing Attestation capabilities.
* DNA and workspace registration are complete.
* Required CI workflows pass.
* Documentation accurately reflects the implemented architecture.
