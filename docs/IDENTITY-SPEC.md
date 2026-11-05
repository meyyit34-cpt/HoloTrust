# HoloTrust Identity Specification

- **Specification:** HT-IDENTITY-001
- **Version:** 0.2.1
- **Status:** Implemented — Identity Discovery complete
- **Related Decision:** ADR-001
- **Scope:** Identity MVP and profile discovery

## 1. Purpose

This document defines the technical behavior of the HoloTrust Identity module.

The module provides a decentralized profile system for Holochain agents representing humans, AI agents, and software services.

It associates public profiles with cryptographic Holochain agent identities and makes those profiles discoverable through DHT links.

A published profile does not establish legal identity and does not guarantee the truthfulness of profile claims.

## 2. Design Principles

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
