HoloTrust Trust Graph Specification

* Specification: HT-TRUST-001
* Version: 0.1.0
* Status: Proposed for Approval
* Scope: Agent-centric, directional, context-aware trust relationships
* Depends on: HT-IDENTITY-001
* Related modules: Identity, Attestation, Reputation
* Architecture decision: ADR-004

1. Purpose

The Trust Graph module enables Holochain agents to declare, discover, and manage explicit trust relationships with other agents.

Trust relationships are directional declarations made by an authenticated agent.

A trust relationship does not independently prove that its subject is honest, safe, competent, or reliable.

The module represents declared relationships, not objective trustworthiness.

2. Core Principles

1. Every relationship has an authenticated author.
2. Relationships are directional.
3. A relationship does not imply reciprocal trust.
4. Relationships are scoped to a defined context.
5. Only the original author may remove their relationship.
6. Relationship history must remain auditable.
7. Sensitive personal information must not be stored in public DHT entries.
8. Trust relationships must not create a universal trust score.
9. Reputation and Attestation remain separate modules.
10. DHT discovery must account for eventual consistency.
11. Historical records must not be silently overwritten or deleted.

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

5. Data Models

5.1 TrustRelationship

Conceptual relationship structure:

pub struct TrustRelationship {
    pub schema_version: u8,
    pub subject_agent: AgentPubKey,
    pub context: TrustContext,
}

The author MUST NOT be supplied as a user-controlled entry field.

The author is derived from the authenticated action.

The relationship target is represented by subject_agent.

5.2 TrustContext

The implementation MUST provide a stable enum or equivalent validated representation for the seven supported context identifiers.

Unknown context values MUST be rejected.

5.3 TrustRelationshipRemoval

Conceptual removal structure:

pub struct TrustRelationshipRemoval {
    pub schema_version: u8,
    pub relationship_action: ActionHash,
}

The removal author MUST be the original relationship author.

A removal record MUST reference the original relationship action.

The removal record MUST be immutable.

5.4 Explicit exclusions

Version 0.1 MUST NOT include:

* A numerical trust score.
* A universal trust ranking.
* An automatic reciprocal relationship.
* An automatic transitive trust calculation.
* Sensitive personal information.
* Attestation evidence fields.
* Reputation evidence fields.

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

Integrity validation MUST enforce the security-critical rules.

A relationship created after a previous relationship was removed MUST be treated as a new declaration.

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

A second removal record for the same relationship MUST be rejected.

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

The system MUST support checking directional and mutual relationships.

Mutual relationships MUST require two independently created active relationships between the same agents in opposite directions and within the same context.

Discovery links MUST be validated by the integrity zome.

Historical records MUST remain retrievable even if discovery links are removed.

Results MUST reflect records available to the querying agent at query time.

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
* Removal records that do not reference a valid relationship.

Public relationship records MUST NOT contain:

* Private keys.
* Passwords.
* Government identity numbers.
* Private contact information.
* Confidential explanations.

The system MUST NOT treat a trust relationship as proof of legal identity or objective trustworthiness.

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

Future evidence-aware integrations require a separate architectural decision.

11. Frontend Requirements

The interface should allow agents to:

* Find another agent.
* Establish a trust relationship.
* Select a trust context.
* View relationships they have established.
* View relationships established by other agents.
* Identify mutual relationships.
* Remove relationships they created.
* Inspect relationship history.

The interface MUST clearly distinguish incoming and outgoing relationships.

The interface MUST NOT display a universal trust score.

The frontend MUST use the real Holochain conductor integration rather than permanent mock data.

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
* Mutual relationship queries.
* Multi-agent discovery.
* Context-specific query behavior.
* DHT consistency behavior.
* Regression compatibility with Identity, Attestation, and Reputation.

Multi-agent tests SHOULD use at least three independent agents.

Static checks and focused tests MUST be completed before expensive multi-conductor integration tests.

Fast CI MUST pass before Full CI is started.

13. Implementation Sequence

1. Approve HT-TRUST-001.
2. Accept ADR-004.
3. Inspect existing workspace, DNA manifests, and coordinator patterns.
4. Create trust_integrity.
5. Create trust coordinator.
6. Implement relationship entry and removal record validation.
7. Implement discovery links.
8. Implement coordinator operations.
9. Add integrity and focused coordinator tests.
10. Register both zomes in Cargo workspace and DNA manifest.
11. Add multi-agent integration tests.
12. Integrate Trust Graph into the web interface.
13. Verify static checks and focused tests.
14. Verify Fast CI.
15. Run Full CI only after all preliminary checks pass.
16. Update architecture, roadmap, and project continuity documentation.

14. Definition of Done

* [ ]	Trust specification approved.
* [ ]	Trust architecture decision accepted.
* [ ]	Trust integrity zome compiles.
* [ ]	Trust coordinator compiles.
* [ ]	Directional relationship creation works.
* [ ]	Context validation works.
* [ ]	Self-relationships are rejected.
* [ ]	Duplicate active relationships are rejected.
* [ ]	Authorized removal works.
* [ ]	Unauthorized removal is rejected.
* [ ]	Duplicate removal is rejected.
* [ ]	Historical relationships remain retrievable.
* [ ]	Mutual relationship queries work.
* [ ]	Multi-agent discovery works.
* [ ]	Frontend displays incoming and outgoing relationships.
* [ ]	No universal trust score is introduced.
* [ ]	Fast CI passes.
* [ ]	Full CI pipeline passes.

⸻

HoloTrust principle

Identity establishes the agent.

Attestation records the claim.

Evidence supports verification.

Reputation evaluates evidence in context.

Trust Graph records declared relationships between agents.
