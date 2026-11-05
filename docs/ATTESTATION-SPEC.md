
# HoloTrust Attestation Specification

- Specification: HT-ATTESTATION-001
- Version: 0.1.0
- Status: Draft — Design Target
- Scope: Verifiable, agent-centric attestations
- Depends on: HT-IDENTITY-001
- Related modules: Identity, Reputation

## 1. Purpose

The Attestation module enables Holochain agents to publish
cryptographically attributable claims about other agents.

An attestation records:
- Who issued the claim.
- Which agent the claim concerns.
- What type of claim is being made.
- The claim description.
- Optional evidence references.
- Optional expiry information.
- Its revocation or supersession history.

An attestation proves the origin and integrity of a published
claim. It does not independently prove that the claim is true.

## 2. Core Principles

The implementation MUST:

1. Derive the issuer from the authenticated Holochain action author.
2. Never trust a user-supplied issuer field.
3. Identify the subject using AgentPubKey.
4. Preserve historical records.
5. Use immutable revocation and supersession records.
6. Support explicit schema versioning.
7. Validate all entries in the integrity zome.
8. Keep sensitive personal information out of public DHT entries.
9. Preserve compatibility with existing Identity records.
10. Separate claim verification from reputation calculation.

## 3. Participant Relationships

An attestation has two principal participants:

- Issuer: The agent publishing and signing the claim.
- Subject: The agent to whom the claim refers.

The issuer is derived from the authenticated action author.

The subject is explicitly declared in the attestation entry.

An agent MAY issue a claim about itself. Such a claim MUST be
identifiable as self-issued and MUST NOT be presented as an
independent third-party verification.

## 4. Claim Types

Version 0.1 defines these stable claim categories:

- Skill
- Experience
- ServiceCompletion
- Membership
- Credential
- Other

Unknown claim types MUST be rejected.

Future claim categories require an explicit schema or protocol update.

## 5. Attestation Data Model

Conceptual structure:

```rust
pub struct Attestation {
    pub schema_version: u8,
    pub subject: AgentPubKey,
    pub claim_type: ClaimType,
    pub claim: String,
    pub evidence: Vec<EvidenceReference>,
    pub expires_at: Option<Timestamp>,
    pub supersedes: Option<ActionHash>,
}
```

The issuer MUST NOT be supplied as an entry field.
It is determined from the authenticated Holochain action.

The authoritative issuance time is the timestamp of the
Holochain action.

### Field constraints

- schema_version: MUST equal the supported version.
- subject: MUST be a valid AgentPubKey.
- claim_type: MUST be a supported category.
- claim: MUST be non-empty and limited to 1000 characters.
- evidence: MUST have a bounded number of references.
- expires_at: Optional; if present, it MUST be later than issuance.
- supersedes: Optional reference to an earlier attestation.

All text limits MUST be tested with Unicode input.

## 6. Evidence References

Evidence references provide traceability without requiring
sensitive documents to be stored in public DHT entries.

Version 0.1 supports references to:

- Holochain ActionHash.
- Holochain EntryHash.
- External HTTPS resources accompanied by a content digest.

An external reference is not automatically trusted.

The system MUST distinguish between:
- A reference being structurally valid.
- A referenced record being available.
- The referenced content matching its digest.
- The referenced content proving the claim.

These are separate verification outcomes.

The initial implementation MUST NOT store private documents,
identity documents, credentials, passwords, or secret data
in public attestation entries.

## 7. Revocation

Published attestations MUST NOT be edited or deleted.

Revocation MUST be represented by a separate immutable record.

Conceptual structure:

```rust
pub struct AttestationRevocation {
    pub schema_version: u8,
    pub attestation: ActionHash,
    pub reason: Option<String>,
}
```

Rules:

1. Only the original issuer may revoke their attestation.
2. A revocation MUST reference an existing attestation.
3. A revoked attestation remains historically retrievable.
4. Duplicate revocation records MUST be rejected.
5. Revocation MUST NOT erase the original claim.

The revocation author is derived from the authenticated action.

## 8. Supersession

Supersession represents a newer claim replacing an older claim
from the same issuer.

Rules:

1. Supersession MUST be represented by a new attestation.
2. The new attestation MUST reference the previous ActionHash.
3. Only the original issuer may supersede their own claim.
4. The previous record MUST remain retrievable.
5. Supersession MUST NOT modify or delete historical data.
6. Circular supersession references MUST be rejected.

A different issuer publishing a contradictory claim does not
supersede the original claim.

## 9. Status Resolution

Attestation status MUST be calculated from immutable records.

Possible states:

- Active
- Revoked
- Expired
- Superseded

The system MUST NOT rely on a mutable status field inside
the original attestation.

Status resolution MUST be deterministic and based on
validated historical records.

## 10. Integrity Zome Responsibilities

The attestation_integrity zome MUST validate:

- Supported schema version.
- Claim type.
- Subject validity.
- Claim length.
- Evidence reference limits and structure.
- Expiry constraints.
- Issuer ownership for revocation.
- Issuer ownership for supersession.
- Referenced attestation existence.
- Duplicate revocation prevention.
- Supersession relationship validity.

Coordinator validation alone is insufficient.

## 11. Coordinator Zome Responsibilities

The attestation coordinator MUST provide operations for:

- create_attestation
- get_attestation
- get_attestations_for_subject
- get_attestations_by_issuer
- revoke_attestation
- verify_attestation
- resolve_attestation_status

All operations MUST use the pinned and verified HDK API.

The coordinator MUST NOT bypass integrity validation.

## 12. Discovery and Link Strategy

The design will support deterministic discovery by:

- Subject agent.
- Issuer agent.
- Attestation type.

Link creation and deletion rules MUST be validated by
the integrity zome.

Historical attestation records MUST remain accessible
even when discovery links change.

## 13. Reputation Separation

Attestations are evidence records.

They are NOT reputation scores.

The Reputation module may later evaluate attestations
within specific domains.

The Attestation module MUST NOT:
- Calculate global reputation.
- Assign universal trust scores.
- Automatically treat every issuer as equally reliable.
- Convert an unverified claim into a verified fact.

## 14. Testing Requirements

The implementation MUST test:

### Valid operations
- Create a third-party attestation.
- Create a self-issued attestation.
- Retrieve an attestation.
- Retrieve attestations by subject.
- Retrieve attestations by issuer.
- Verify issuer attribution.
- Revoke an attestation.
- Supersede an attestation.
- Resolve active and historical status.

### Invalid operations
- Unsupported schema version.
- Unknown claim type.
- Empty claim.
- Claim exceeding length limit.
- Invalid evidence reference.
- Expiry earlier than issuance.
- Unauthorized revocation.
- Unauthorized supersession.
- Duplicate revocation.
- Missing referenced attestation.
- Circular supersession.

### Multi-agent tests

At least three independent agents SHOULD be used to verify:

- Issuer attribution.
- Subject discovery.
- Unauthorized revocation rejection.
- Unauthorized supersession rejection.
- Historical record preservation.
- Cross-agent retrieval.

## 15. Implementation Sequence

1. Approve this specification.
2. Record the architectural decision.
3. Create attestation_integrity zome.
4. Create attestation coordinator zome.
5. Add DNA manifest entries.
6. Implement entry and link validation.
7. Implement coordinator operations.
8. Add multi-agent integration tests.
9. Verify revocation and supersession.
10. Run the complete CI pipeline.

Existing Identity zomes MUST remain unchanged during
the initial Attestation implementation.

## 16. Definition of Done

- [ ] Specification approved.
- [ ] Architectural decision documented.
- [ ] Attestation integrity zome compiles.
- [ ] Attestation coordinator compiles.
- [ ] Issuer attribution is verified.
- [ ] Subject discovery works.
- [ ] Evidence references are validated.
- [ ] Revocation is auditable.
- [ ] Supersession is auditable.
- [ ] Multi-agent tests pass.
- [ ] Existing Identity tests remain green.
- [ ] Full CI pipeline passes.

---

HoloTrust principle:

Identity establishes the agent.
Attestation establishes the signed claim.
Evidence supports verification.
Reputation evaluates evidence in context.
