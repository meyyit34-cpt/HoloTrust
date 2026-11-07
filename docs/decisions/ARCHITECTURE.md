ADR-004: Trust Graph Architecture

* Status: Proposed
* Date: 2026-11-07
* Decision: Separate, directional Trust Graph module
* Related specification: HT-TRUST-001
* Related decisions: ADR-001, ADR-002, ADR-003

1. Context

HoloTrust requires an explicit relationship system that allows agents to declare trust relationships with other agents.

Identity establishes an agent.
Attestation records attributable claims.
Reputation evaluates evidence within a context.
Trust Graph records declared relationships between agents.

Trust relationships must not be confused with objective proof of reliability or reputation.

The existing Identity, Attestation, and Reputation modules are functional and must remain independently responsible for their respective data and validation rules.

2. Decision

Trust Graph MUST be implemented as a separate Holochain zome pair:

* trust_integrity
* trust

Existing Identity, Attestation, and Reputation zomes MUST NOT become the implementation location for Trust Graph functionality.

Trust Graph MUST NOT modify, rewrite, or reinterpret records belonging to other modules.

3. Directional Relationships

Every trust relationship MUST have:

* An authenticated author.
* A target AgentPubKey.
* A supported trust context.
* A creation action that remains historically retrievable.

The author MUST be derived from the authenticated Holochain action author.

The author MUST NOT be supplied as a trusted user-controlled entry field.

Relationships MUST be directional.

A relationship from Agent A to Agent B MUST NOT automatically create a relationship from Agent B to Agent A.

Self-directed relationships MUST be rejected.

4. Contextual Trust

Version 0.1 MUST support these stable context identifiers:

* general
* commerce
* professional
* hospitality
* technical
* community
* reliability

Unknown contexts MUST be rejected by the integrity layer.

A relationship in one context MUST NOT automatically apply to another context.

The initial implementation MUST NOT calculate a universal trust score.

5. Immutable Relationship History

Published relationships MUST NOT be edited or deleted.

Removing a relationship MUST create a separate immutable removal record.

Only the original relationship author may remove that relationship.

The removal record MUST reference the original relationship and MUST be uniquely associated with it.

Unauthorized removal attempts MUST be rejected by integrity validation.

Historical relationship and removal records MUST remain retrievable.

A new relationship after removal MUST be treated as a new declaration.

6. Integrity and Coordinator Separation

The trust_integrity zome MUST enforce all consensus-critical validation rules.

The trust coordinator MUST provide application-facing operations without bypassing integrity validation.

Validation MUST include:

* Supported schema version.
* Valid target AgentPubKey.
* Supported context.
* No self-relationships.
* Duplicate active relationship rejection.
* Valid removal references.
* Removal authorization.
* Duplicate removal rejection.

Security MUST NOT depend solely on coordinator-side or frontend validation.

7. Discovery and Queries

The coordinator MUST support:

* create_trust_relationship
* remove_trust_relationship
* get_trust_relationship
* get_trust_relationships_by_author
* get_trust_relationships_for_subject
* get_trust_relationships_by_context
* get_trust_relationship_history

Queries MUST distinguish active and removed relationships.

Discovery links MUST be validated by the integrity zome.

Historical records MUST remain retrievable independently of active discovery links.

The API MUST document Holochain DHT eventual consistency and MUST NOT claim globally instantaneous discovery.

8. Module Boundaries

Identity

Responsible for agent identity, profile creation, and profile discovery.

Attestation

Responsible for attributable claims, evidence references, revocation, and supersession.

Reputation

Responsible for contextual evaluation of evidence.

Trust Graph

Responsible for explicit, directional, context-aware relationships between agents.

Trust Graph MUST NOT modify Attestation or Reputation records.

A trust relationship MUST NOT automatically be interpreted as an attestation, reputation result, or proof of trustworthiness.

9. Privacy and Security

Public relationship records MUST NOT contain sensitive personal information.

Private keys, passwords, government identity numbers, private contact information, and confidential explanations MUST NOT be stored in public DHT entries.

The system MUST NOT introduce automatic reciprocal trust or automatic transitive trust propagation.

10. Testing Requirements

Tests MUST cover:

* Valid relationship creation.
* Authenticated author attribution.
* Directional relationship behavior.
* Supported and unsupported contexts.
* Self-relationship rejection.
* Duplicate active relationship rejection.
* Authorized removal.
* Unauthorized removal rejection.
* Duplicate removal rejection.
* Historical record preservation.
* Active and removed relationship queries.
* Multi-agent discovery.
* Context-specific queries.
* DHT eventual consistency behavior.

Multi-agent integration tests SHOULD use at least three independent agents.

11. Implementation Order

1. Approve HT-TRUST-001.
2. Accept this architectural decision.
3. Create trust_integrity.
4. Create trust coordinator.
5. Register both zomes in the workspace and DNA manifest.
6. Implement relationship and removal entry validation.
7. Implement discovery links.
8. Implement coordinator operations.
9. Add integrity and multi-agent integration tests.
10. Integrate Trust Graph into the web interface.
11. Run Fast CI and targeted tests.
12. Run Full CI only after preliminary checks pass.

Existing Identity, Attestation, and Reputation functionality MUST remain compatible throughout implementation.

12. Consequences

Positive:

* Trust relationships remain independent from reputation.
* Directional declarations are explicit.
* Context-specific relationships are supported.
* Historical changes remain auditable.
* Integrity validation protects relationship ownership and removal authorization.

Trade-offs:

* A separate zome pair increases implementation complexity.
* Multi-agent discovery requires integration testing.
* DHT results remain eventually consistent.
* Relationship history requires additional storage and query logic.

13. Final Decision

Proposed architectural separation:

Identity establishes the agent.

Attestation records attributable claims.

Evidence supports verification.

Reputation evaluates evidence in context.

Trust Graph records declared relationships between agents.

No universal trust score or automatic transitive trust calculation is introduced.
