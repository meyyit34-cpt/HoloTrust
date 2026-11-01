HoloTrust Identity Specification

* Specification: HT-IDENTITY-001
* Version: 0.1.0
* Status: Draft — Implementation Target
* Related Decision: ADR-001
* Scope: Identity MVP

1. Purpose

This document defines the initial technical requirements for the HoloTrust Identity module.

The module provides a decentralized profile system for Holochain agents representing humans, AI agents, and software services.

It establishes cryptographic identity association and public profile discovery.

It does not establish legal identity or guarantee the truthfulness of profile claims.

2. Design Principles

The implementation MUST:

1. Use the Holochain agent identity as the cryptographic identity anchor.
2. Associate each profile with its author.
3. Validate profile data at the integrity layer.
4. Keep sensitive personal information out of public DHT entries.
5. Support schema versioning.
6. Reject invalid or unauthorized operations.
7. Preserve the distinction between identity and verification.

3. Participant Types

The initial version supports:

pub enum AgentType {
    Human,
    AI,
    Service,
}

These values represent declared participant categories only.

They are not independently verified classifications.

4. Profile Data Model

The conceptual profile structure is:

pub struct AgentProfile {
    pub schema_version: u8,
    pub display_name: String,
    pub agent_type: AgentType,
    pub description: Option<String>,
}

Field Constraints

Field	Requirement
schema_version	Must equal the supported initial version
display_name	Required, non-empty, maximum 64 characters
agent_type	Must be a supported enum variant
description	Optional, maximum 280 characters

Length validation must be defined consistently and tested, including Unicode input.

The implementation must not accept empty or whitespace-only display names.

5. Identity Association

A profile must be associated with its author.

The publishing agent is determined by the authenticated Holochain action context, not by a user-supplied author field.

The initial MVP will allow one current profile per agent.

The profile retrieval interface must support lookup by AgentPubKey.

6. Required Operations

6.1 create_profile

Creates a new profile for the calling agent.

Requirements:

* Validate all fields.
* Reject invalid data.
* Associate the profile with the authenticated author.
* Prevent accidental duplicate initial profile creation.
* Return a useful result or explicit error.

6.2 get_my_profile

Retrieves the current agent’s profile.

Requirements:

* Use the current agent’s identity.
* Return the profile if one exists.
* Return a clear not-found result otherwise.

6.3 get_profile

Retrieves another agent’s public profile using an AgentPubKey.

Requirements:

* Validate the lookup input.
* Resolve the profile through the selected discovery mechanism.
* Return a clear not-found result if no profile exists.

7. Integrity Layer

The Integrity Zome is responsible for enforcing data rules independently of coordinator workflows.

It must validate:

* Entry structure.
* Schema version.
* Display name constraints.
* Description constraints.
* Supported participant type.
* Entry author association.
* Link and update constraints, where applicable.

Coordinator functions must never be treated as a substitute for integrity validation.

8. Coordinator Layer

The Coordinator Zome provides application-facing functions.

It is responsible for:

* Profile creation workflows.
* Profile retrieval.
* Current-agent profile lookup.
* Discovery link creation and retrieval.
* Clear error handling.

The exact Holochain API signatures must follow the pinned and verified HDK version.

9. Discovery Strategy

The initial discovery design will use a deterministic link from an agent identity to its profile entry.

The final link base and path representation must be selected during implementation.

The design must ensure:

* One current profile can be resolved per agent.
* Duplicate or conflicting links are handled safely.
* Updates do not silently create ambiguous profile states.
* Retrieval behavior is deterministic.

The implementation must document how stale links and profile updates are handled.

10. Security Requirements

The MVP must consider:

* Forged author claims.
* Unauthorized profile updates.
* Invalid entry data.
* Oversized input.
* Malformed lookup keys.
* Duplicate creation attempts.
* Unexpected schema versions.
* Malicious or misleading profile descriptions.

No private keys or credentials may be stored in profile entries.

No civil identity verification is implied by profile publication.

11. Testing Requirements

The implementation must include tests for:

Valid Operations

* Create a Human profile.
* Create an AI profile.
* Create a Service profile.
* Retrieve the calling agent’s profile.
* Retrieve another agent’s profile.

Invalid Operations

* Empty display name.
* Whitespace-only display name.
* Display name exceeding the limit.
* Description exceeding the limit.
* Unsupported schema version.
* Unauthorized profile modification.
* Duplicate profile creation.
* Missing profile lookup.

Multi-Agent Tests

At least two independent agents must be used to verify:

* Profile ownership.
* Profile discovery.
* Retrieval across agents.
* Unauthorized modification rejection.

12. Out of Scope for Version 0.1

The following are explicitly excluded:

* Global reputation scores.
* Trust scoring algorithms.
* Economic transactions.
* HoloFuel integration.
* hREA integration.
* Civil identity verification.
* Identity recovery.
* Key rotation.
* Delegated authority.
* Private profile storage.

These features may be addressed in later specifications.

13. Definition of Done

Identity MVP 0.1 is complete only when:

* [ ]	Compatible Holochain toolchain is pinned.
* [ ]	Integrity Zome compiles.
* [ ]	Coordinator Zome compiles.
* [ ]	Profile creation works.
* [ ]	Profile retrieval works.
* [ ]	Ownership validation works.
* [ ]	Invalid inputs are rejected.
* [ ]	Multi-agent tests pass.
* [ ]	Documentation matches the implemented behavior.

A design document or untested code is not considered a completed implementation.

⸻

HoloTrust Identity MVP 0.1

Identity is the foundation. Verification is evidence. Reputation is contextual.
