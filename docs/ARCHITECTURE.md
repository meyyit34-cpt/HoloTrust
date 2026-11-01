HoloTrust — Technical Architecture

1. Overview

HoloTrust is a modular, agent-centric decentralized identity and reputation application built on Holochain.

The architecture separates identity, attestations, reputation, and trust relationships into independent functional layers.

2. Architectural Principles

* Agent-centric data ownership.
* Source-chain validation and integrity.
* Distributed application validation through Holochain.
* Privacy-first data design.
* Modular and independently testable zomes.
* Explicit schema versioning.
* Evidence-based reputation.

3. System Layers

Layer 1 — Identity

Responsible for participant profiles and identity discovery.

Initial capabilities:

* Create profile.
* Retrieve own profile.
* Retrieve another agent’s profile.
* Update profile.
* Validate profile ownership.

Layer 2 — Attestations

Responsible for signed claims issued by participants.

Planned capabilities:

* Create attestation.
* Retrieve attestation.
* Revoke or supersede an attestation.
* Verify issuer and evidence references.

Layer 3 — Reputation

Responsible for contextual reputation derived from attestations.

Principles:

* No universal reputation score.
* Domain-specific evaluations.
* Evidence traceability.
* Transparent calculation rules.

Layer 4 — Trust Graph

Responsible for relationships between agents.

Planned capabilities:

* Establish trust relationship.
* Remove relationship.
* Query relationships.
* Represent trust context.

Layer 5 — Economic Integration

Future integration with Holochain economic infrastructure, including hREA and potentially Unyt.

This layer is explicitly outside the initial MVP.

4. Holochain DNA Structure

Initial DNA name:

holotrust

Planned zomes:

* identity_integrity
* identity
* attestation_integrity
* attestation
* reputation_integrity
* reputation
* trust_integrity
* trust

The MVP begins with the Identity zome pair only.

5. Identity Data Model

Initial conceptual profile:

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

The profile must be linked to its authoring agent and validated by the integrity zome.

6. Profile Discovery

Profiles will be discoverable through Holochain links.

The implementation must define:

* A stable discovery anchor.
* Link creation and deletion rules.
* Duplicate profile handling.
* Update semantics.
* Validation of link targets.

7. Validation and Security

Integrity rules must verify:

* Supported schema version.
* Bounded text lengths.
* Valid agent type.
* Author ownership.
* Valid entry and link relationships.

Sensitive personal information must never be required for a public profile.

8. Testing Strategy

Testing will include:

* Unit tests.
* Integrity validation tests.
* Coordinator function tests.
* Multi-agent integration tests.
* Invalid input tests.
* Profile update and discovery tests.

9. Development Sequence

Sprint 0:

* Repository setup.
* Toolchain compatibility.
* DNA scaffold.
* CI configuration.

Sprint 1:

* Identity integrity zome.
* Identity coordinator zome.
* Profile creation and retrieval.
* Automated tests.

Sprint 2:

* Profile updates.
* Discovery links.
* Multi-agent validation.

Sprint 3:

* Attestation architecture.

10. Architecture Decision

HoloTrust will not implement reputation as a single global score.

Trust must remain contextual, explainable, and connected to verifiable evidence.

⸻

HoloTrust — Technical Foundation for Decentralized Trust.
