HoloTrust Trust Graph Specification

* Specification: HT-TRUST-001
* Version: 0.1.0
* Status: Draft — Design Target
* Scope: Agent-centric, context-aware trust relationships
* Depends on: HT-IDENTITY-001
* Related modules: Identity, Attestation, Reputation

1. Purpose

The Trust Graph module enables Holochain agents to declare, discover, and manage explicit trust relationships with other agents.

Trust relationships are directional declarations made by an authenticated agent.

A trust relationship does not independently prove that its subject is honest, safe, competent, or reliable.

The module represents declared relationships, not objective trustworthiness.

2. Core Principles

1. Every relationship has an authenticated author.
2. Relationships are directional.
3. A relationship does not imply reciprocal trust.
4. Relationships may be scoped to a specific context.
5. Only the original author may remove their relationship.
6. Relationship history must remain auditable.
7. Sensitive personal information must not be stored in public DHT entries.
8. Trust relationships must not create a universal trust score.
9. Reputation and Attestation remain separate modules.
10. DHT discovery must account for eventual consistency.

3. Relationship Model

A relationship represents:

* The agent establishing the relationship.
* The target agent.
* The trust context.
* The authenticated creation action.
* Its historical removal state, if applicable.

The author MUST be derived from the authenticated Holochain action author.

The target MUST be represented by AgentPubKey.

An agent MUST NOT establish a trust relationship with itself.

Relationships MUST be directional.

A relationship from Agent A to Agent B MUST NOT automatically create a relationship from Agent B to Agent A.

4. Trust Contexts

Version 0.1 defines these stable context identifiers:

* general
* commerce
* professional
* hospitality
* technical
* community
* reliability

Unknown contexts MUST be rejected.

Context identifiers are protocol values and MUST remain stable.

Display labels may be localized.

A relationship in one context MUST NOT automatically apply to another context.

5. Data Model

Conceptual relationship structure:

pub struct TrustRelationship {
    pub schema_version: u8,
    pub subject_agent: AgentPubKey,
    pub context: TrustContext,
}

The author MUST NOT be supplied as a user-controlled entry field.

The author is derived from the authenticated action.

The initial implementation MUST NOT include:

* A numerical trust score.
* A universal trust ranking.
* An automatic reciprocal relationship.
* An automatic transitive trust calculation.
* Sensitive personal information.

6. Creation Rules

The integrity layer MUST validate:

* Supported schema version.
* Valid subject AgentPubKey.
* Supported trust context.
* Author and subject are different agents.
* Duplicate active relationships are rejected.
* Relationship author is derived from the authenticated action.

A relationship MUST be discoverable by its author and target.

Coordinator validation alone is insufficient.

7. Removal and Historical Records

Removing a relationship MUST create a separate immutable removal record.

Only the original relationship author may remove that relationship.

A removal record MUST:

* Reference an existing relationship.
* Be authored by the original relationship author.
* Be unique for that relationship.
* Preserve the original relationship.
* Remain historically retrievable.

The original relationship MUST NOT be edited or deleted.

A removed relationship MUST NOT be presented as active.

A new relationship after removal is a new historical declaration, not an edit of the previous relationship.

8. Discovery and Queries

The coordinator MUST support:

* create_trust_relationship
* remove_trust_relationship
* get_trust_relationship
* get_trust_relationships_by_author
* get_trust_relationships_for_subject
* get_trust_relationships_by_context
* get_trust_relationship_history

Queries MUST distinguish active relationships from removed relationships.

Discovery links MUST be validated by the integrity zome.

Historical records MUST remain retrievable even if discovery links are removed.

Results MUST reflect the records available to the querying agent at query time.

The API MUST NOT claim globally instantaneous DHT consistency.

9. Security and Privacy

The integrity layer MUST reject:

* Unsupported schema versions.
* Unknown trust contexts.
* Self-directed relationships.
* Invalid subject references.
* Unauthorized removal attempts.
* Duplicate active relationships.
* Duplicate removal records.
* Invalid relationship references.

Public relationship records MUST NOT contain private keys, passwords, government identity numbers, private contact information, or confidential explanations.

10. Module Boundaries

Identity

Responsible for cryptographic agent identity and public profile discovery.

Attestation

Responsible for attributable claims and their historical verification.

Reputation

Responsible for contextual evaluation of evidence.

Trust Graph

Responsible for explicit, directional relationships between agents.

The Trust Graph MUST NOT modify Attestation or Reputation records.

A trust relationship MUST NOT automatically be interpreted as an attestation or reputation result.

11. Frontend Requirements

The interface should allow agents to:

* Find another agent.
* Establish a trust relationship.
* Select a trust context.
* View relationships they have established.
* View relationships established by other agents.
* Remove relationships they created.
* Inspect relationship history.

The interface MUST clearly distinguish incoming and outgoing relationships.

The interface MUST NOT display a universal trust score.

12. Testing Requirements

Tests MUST cover:

* Valid relationship creation.
* Author attribution.
* Directional relationship behavior.
* Supported context acceptance.
* Unknown context rejection.
* Self-relationship rejection.
* Duplicate active relationship rejection.
* Authorized removal.
* Unauthorized removal rejection.
* Duplicate removal rejection.
* Historical record preservation.
* Active and removed relationship queries.
* Multi-agent discovery.
* Context-specific query behavior.
* DHT consistency behavior.

Multi-agent tests SHOULD use at least three independent agents.

13. Implementation Sequence

1. Approve HT-TRUST-001.
2. Record the Trust Graph architectural decision.
3. Create trust_integrity.
4. Create trust coordinator.
5. Register both zomes in the workspace and DNA manifest.
6. Implement relationship entry and removal record validation.
7. Implement discovery links.
8. Implement coordinator operations.
9. Add integrity and multi-agent integration tests.
10. Integrate Trust Graph into the web interface.
11. Verify the complete CI pipeline and real-conductor behavior.

14. Definition of Done

* [ ]	Trust specification approved.
* [ ]	Trust architecture decision documented.
* [ ]	Trust integrity zome compiles.
* [ ]	Trust coordinator compiles.
* [ ]	Directional relationship creation works.
* [ ]	Context validation works.
* [ ]	Duplicate active relationships are rejected.
* [ ]	Authorized removal works.
* [ ]	Unauthorized removal is rejected.
* [ ]	Historical relationships remain retrievable.
* [ ]	Multi-agent discovery works.
* [ ]	Frontend displays incoming and outgoing relationships.
* [ ]	No universal trust score is introduced.
* [ ]	Full CI pipeline passes.

⸻

HoloTrust principle:

Identity establishes the agent.
Attestation records the claim.
Evidence supports verification.
Reputation evaluates evidence in context.
Trust Graph records declared relationships.
