
# HoloTrust Identity Specification

- **Specification:** HT-IDENTITY-001
- **Version:** 0.2.0
- **Status:** Implemented MVP — Discovery behavior documented
- **Related Decision:** ADR-001
- **Scope:** Identity MVP and profile discovery

## 1. Purpose

This document defines the technical behavior of the HoloTrust Identity module.

The module provides a decentralized profile system for Holochain agents representing humans, AI agents, and software services.

It associates public profiles with cryptographic Holochain agent identities and makes those profiles discoverable through DHT links.

A published profile does not establish legal identity and does not guarantee the truthfulness of profile claims.

## 2. Design Principles

The implementation follows these principles:

1. Holochain agent identity is the cryptographic identity anchor.
2. Profile ownership is derived from the authenticated action author.
3. Profile data is validated by the integrity zome.
4. Sensitive personal information must not be stored in public DHT entries.
5. Profile entries use explicit schema versioning.
6. Invalid and unauthorized operations are rejected.
7. Identity publication is distinct from verification.
8. Updates preserve historical records while changing the current discovery index.

## 3. Participant Types

The initial version supports:

```rust
pub enum AgentType {
    Human,
    Ai,
    Service,
}
```

The `Ai` variant is serialized as `AI`.

These values represent declared participant categories. They are not independently verified classifications.

## 4. Profile Data Model

The profile entry is:

```rust
pub struct AgentProfile {
    pub schema_version: u8,
    pub display_name: String,
    pub agent_type: AgentType,
    pub description: Option<String>,
}
```

### Field constraints

| Field | Requirement |
|---|---|
| schema_version | Must equal 1 |
| display_name | Required, 1–64 characters after trimming |
| agent_type | Must be a supported enum variant |
| description | Optional, maximum 280 characters |

Control characters are rejected in display names and descriptions.

Length validation counts Unicode characters.

## 5. Identity Association

A profile is associated with the Holochain agent that authored its creation action.

The author is obtained from the authenticated action context, not from a user-supplied author field.

The initial implementation permits one current profile per agent.

A second `create_profile` call is rejected when the agent already has a discoverable profile. The error directs the caller to `update_profile`.

## 6. Discovery Architecture

### 6.1 Discovery link

The identity integrity zome defines the `AgentToProfile` link type.

The link uses:

- Base: the owning agent's `AgentPubKey`.
- Target: the profile creation or update `ActionHash`.
- Tag: `profile`.

The coordinator creates this link through `index_profile`.

The integrity zome verifies that:

1. The link base is an agent public key.
2. The base agent matches the link author.
3. The target is an action hash.
4. The target record exists and is valid.
5. The target record author matches the owning agent.
6. The target contains an `AgentProfile`.

### 6.2 Profile discovery

`get_profile` accepts an `AgentPubKey`.

It queries `AgentToProfile` links for that agent and examines the returned links in reverse order.

A candidate is returned only when:

- The link author matches the requested agent.
- The link target is an action hash.
- The target record can be retrieved.
- The target record author matches the requested agent.
- The entry can be decoded as an `AgentProfile`.
- The profile passes `validate_profile`.

Invalid, missing, or unrelated candidates are skipped.

If no valid candidate is found, the function returns `None`.

`get_my_profile` follows the same validation process using the calling agent's initial public key.

### 6.3 Discovery consistency

DHT link availability depends on network propagation and consistency.

The coordinator selects the latest valid candidate among the links returned by its query, but reverse iteration alone does not establish a globally ordered or globally deterministic result across all network states.

Multi-agent tests use `await_consistency` before asserting that another agent can discover an updated profile.

## 7. Profile Update Semantics

The `update_profile` operation requires the action hash of the caller's current profile.

The coordinator:

1. Retrieves the referenced record.
2. Confirms that the record belongs to the calling agent.
3. Confirms that it contains an `AgentProfile`.
4. Checks that the referenced profile is the current discoverable profile.
5. Validates the replacement profile.
6. Creates an update action.
7. Deletes the old profile discovery link authored by the caller.
8. Creates a new discovery link targeting the updated action.

An update targeting an older profile version is rejected as stale.

The previous profile action remains in the source-chain history; removing its discovery link does not erase that historical action.

The current discovery index points to the updated profile after the new link becomes available.

## 8. Required Coordinator Operations

| Operation | Purpose |
|---|---|
| `create_profile` | Create the first profile for the calling agent |
| `get_my_profile` | Retrieve the calling agent's current discoverable profile |
| `get_profile` | Discover another agent's public profile |
| `update_profile` | Update the current profile and refresh its discovery link |

## 9. Integrity and Security

The integrity zome validates profile schema, display name, description, participant type, ownership, and discovery link relationships.

The coordinator additionally checks ownership and current-profile state before updating.

Security requirements include:

- Reject forged ownership claims.
- Reject invalid profile data.
- Reject duplicate initial profile creation.
- Reject unauthorized profile updates.
- Reject stale profile updates.
- Reject discovery links that point to another agent's profile.
- Avoid publishing private keys, credentials, or sensitive personal data.

Coordinator checks do not replace integrity validation.

## 10. Testing Requirements and Current Coverage

The current test suite includes:

- Profile creation and retrieval.
- Human, AI, and Service profile validation.
- Invalid display names and descriptions.
- Unsupported schema versions.
- Duplicate profile creation rejection.
- Profile ownership checks.
- Multi-agent profile discovery.
- Unauthorized update rejection.
- Successful profile updates.
- Stale update rejection.
- Discovery of the updated profile after DHT consistency.

### Remaining Phase 2 verification

- [ ] Add an explicit test for duplicate `AgentToProfile` link handling.
- [ ] Verify conflicting or repeated discovery links do not cause an incorrect profile to be returned.
- [ ] Confirm the behavior through multi-agent tests where applicable.

These items remain open until implemented and tested.

## 11. Out of Scope for Version 0.2

The following are not provided by the Identity module:

- Legal or civil identity verification.
- Global reputation scores.
- Trust scoring algorithms.
- Economic transactions.
- HoloFuel integration.
- hREA integration.
- Identity recovery.
- Key rotation.
- Delegated authority.
- Private profile storage.
- A globally ordered DHT discovery index.

These capabilities require separate specifications and implementation stages.

## 12. Definition of Done

Identity Discovery is complete only when:

- [x] Agent-based profile discovery anchor is implemented.
- [x] Profile creation and retrieval work.
- [x] Profile ownership is validated.
- [x] Invalid profile data is rejected.
- [x] Multi-agent profile discovery is tested.
- [x] Unauthorized profile updates are rejected.
- [x] Stale profile updates are rejected.
- [x] Updated profiles become discoverable after DHT consistency.
- [x] Documentation reflects the implemented discovery and update behavior.
- [ ] Explicit duplicate profile-link handling tests pass.
- [ ] Conflicting link behavior is verified.

---

HoloTrust — Identity is the foundation. Verification is evidence. Reputation is contextual.
