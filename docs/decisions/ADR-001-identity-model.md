ADR-001: HoloTrust Identity Model

* Status: Accepted — Initial Design
* Date: 2026-11-02
* Decision Owners: HoloTrust Project
* Scope: Identity Foundation

1. Context

HoloTrust requires a decentralized identity model capable of representing:

* Human participants.
* Artificial intelligence agents.
* Independent software services.

The identity layer must operate without depending on a central identity provider.

Holochain’s agent-centric architecture provides the foundation for this model.

An agent’s cryptographic identity establishes control of a key, but does not independently prove a real-world identity.

2. Decision

HoloTrust will use the Holochain AgentPubKey as the cryptographic foundation of each participant’s network identity.

Each participant may publish a public profile associated with their AgentPubKey.

The initial identity model will distinguish three participant types:

* Human
* AI
* Service

These types describe the declared category of a participant. They are not proof of legal identity, biological status, or independent verification.

3. Initial Data Model

The initial profile schema is:

pub struct AgentProfile {
    pub schema_version: u8,
    pub display_name: String,
    pub agent_type: AgentType,
    pub description: Option<String>,
}
pub enum AgentType {
    Human,
    AI,
    Service,
}

This is a conceptual schema. Exact Holochain serialization and entry definitions will be finalized during implementation.

Field Definitions

Field	Purpose
schema_version	Enables future schema evolution
display_name	Public display name
agent_type	Declared participant category
description	Optional public description

The profile must not contain private keys, passwords, government identification numbers, or other sensitive personal information.

4. Identity and Verification Separation

HoloTrust will distinguish three concepts:

4.1 Cryptographic Identity

Proves control of a Holochain agent key.

4.2 Declared Profile

Contains information voluntarily published by the agent.

4.3 Verified Attestations

Contains claims made by other agents or authorized verification systems, supported by evidence and provenance.

These concepts must never be treated as interchangeable.

A profile declaring Human does not establish civil identity.

A profile declaring AI does not establish model authenticity.

A profile declaring Service does not establish organizational ownership.

Verification must be represented separately through attestations.

5. Privacy Principles

The identity layer will follow these rules:

* Public DHT data must be designed for permanent disclosure.
* Sensitive personal information must not be published.
* Private information must remain outside public profile entries.
* Optional profile fields should be minimized.
* Identity claims must be independently verifiable where possible.
* No central authority should be required for basic cryptographic identity creation.

6. Profile Ownership and Integrity

The initial implementation must ensure:

* A profile is associated with its publishing agent.
* Unauthorized agents cannot modify another agent’s profile.
* Input fields have defined length limits.
* Unsupported schema versions are rejected or handled explicitly.
* Profile updates preserve a verifiable history where required.

Exact update and revocation semantics will be specified before implementation.

7. Holochain Architecture

The identity foundation will use two logical layers:

Integrity Zome

Responsible for:

* Entry definitions.
* Data validation.
* Field constraints.
* Schema validation.
* Link validation.

Coordinator Zome

Responsible for:

* Profile creation.
* Profile retrieval.
* Profile update workflows.
* Application-level identity operations.

The final zome structure will follow the verified Holochain toolchain and official development conventions.

8. Future Extensions

The initial model is intentionally minimal.

Future versions may introduce:

* Verifiable attestations.
* Agent capability declarations.
* Identity recovery mechanisms.
* Key rotation.
* Delegated agent authority.
* Contextual reputation.
* Trust relationship graphs.
* Human-to-agent and agent-to-agent relationships.

These extensions must not compromise the privacy or security of the identity foundation.

9. Consequences

Positive

* A common identity foundation for humans, AI agents, and services.
* No mandatory centralized identity provider.
* Clear separation between key ownership and real-world verification.
* A minimal and extensible initial data model.
* A foundation for attestations and contextual reputation.

Trade-offs

* Cryptographic identity alone does not establish trust.
* Public profile data requires careful privacy design.
* Key loss and recovery require additional architecture.
* Reputation cannot be safely reduced to a single universal score.

10. Implementation Priority

The first implementation milestone will deliver:

1. Identity profile entry definition.
2. Integrity validation.
3. Profile creation.
4. Profile retrieval.
5. Ownership validation.
6. Automated tests.

No reputation scoring or economic integration will be implemented before the identity foundation is stable.

⸻

Decision Summary

HoloTrust adopts Holochain AgentPubKey as the cryptographic identity foundation and a minimal, public, versioned profile model for humans, AI agents, and services. Real-world verification remains a separate, evidence-based attestation layer.

HoloTrust — Identity first. Trust through verifiable evidence.
