HoloTrust — Project Continuity & Development Handoff

Project: HoloTrust
Repository: https://github.com/meyyit34-cpt/HoloTrust
GitHub owner: Meyyit34
Document purpose: Preserve project direction, architectural decisions, development status, engineering rules, and the next implementation steps across future work sessions.

⸻

1. Project Vision

HoloTrust is an agent-centric, decentralized identity, attestation, reputation, and trust network built on Holochain.

The long-term objective is to provide infrastructure for agents and people to establish identity, make verifiable claims, evaluate evidence contextually, declare trust relationships, and coordinate economic activity without depending on a centralized identity or trust authority.

Core principle:

* Identity establishes the agent.
* Attestation records the claim.
* Evidence supports verification.
* Reputation evaluates evidence in context.
* Trust Graph records declared relationships between agents.
* Economic coordination enables agents to exchange value and resources.

The project must evolve toward a complete, usable application and production-capable network. It is not intended to remain a documentation-only project or a limited prototype.

2. Project Scope — Nothing Is Dropped

All major capabilities remain within the project scope. Future work must be implemented in dependency order, not removed or silently postponed.

The full scope includes:

1. Decentralized agent identity and profile discovery.
2. Verifiable attestations, authorization, revocation, and supersession.
3. Evidence-based contextual reputation.
4. Directional, context-aware Trust Graph.
5. Real Holochain conductor integration and multi-agent operation.
6. Economic coordination and hREA integration.
7. Unyt integration and agent-to-agent value exchange.
8. HoloFuel-compatible economic coordination and relevant infrastructure integration.
9. Complete frontend and usable end-to-end application workflows.
10. Deployment, security hardening, performance, observability, and production readiness.

Economic and infrastructure integrations must be investigated and implemented against their actual technical compatibility. Do not invent APIs or claim integrations work before they are tested.

3. Development Status

Completed

* Phase 0 — Foundation.
* Phase 1 — Identity MVP.
* Phase 2 — Identity Discovery.
* Phase 3 — Verifiable Attestations.
* Phase 4 — Contextual Reputation.

Phase 4 has passed Fast CI and Full CI.

Latest verified successful Full CI:

* Run ID: 42371037589
* URL: https://github.com/meyyit34-cpt/HoloTrust/actions/runs/42371037589
* Commit: 5f681ff1a864312089f13e65d83f6f5145ecd987
* Result: Success.

The successful run included WASM build, artifact generation, Nix checks, native test compilation, DNA/hApp packaging, real-conductor smoke testing, workspace tests, and Fast formatting checks.

The reputation specification was updated to Implemented and its Definition of Done was completed.

Current phase

Phase 5 — Trust Graph

Trust Graph specification and architecture documentation have been prepared. Implementation has not yet started.

4. Trust Graph — Current Architecture

Specification:
docs/TRUST-SPECIFICATION.md

Current status: Proposed for Approval

Latest verified SHA:
d98bf5b612c41cca4cc8dce62e486a9a96fe2dbe

Architecture decision:
docs/decisions/ADR-004-TRUST-GRAPH-ARCHITECTURE.md

Current status: Proposed

ADR-004 defines a dedicated Trust Graph domain with:

* trust_integrity
* trust coordinator

Version 0.1 represents explicit agent-declared trust relationships only.

Mandatory Trust Graph rules

* Relationships are directional.
* Trust is context-specific.
* A relationship from A to B never automatically creates B to A.
* Mutual trust exists only when both agents independently create active relationships in the same context.
* Self-directed relationships are invalid.
* Only the original author may remove their relationship.
* Removal creates a separate immutable historical record.
* Original relationship records must not be edited or deleted.
* Duplicate active relationships and duplicate removal records must be rejected.
* No universal trust score or ranking.
* No automatic transitive trust calculation.
* No evidence fields in Trust Graph v0.1.
* Attestation and Reputation remain separate domains.
* Public DHT records must not contain sensitive personal information.
* DHT eventual consistency must be represented honestly.

Supported context identifiers

* general
* commerce
* professional
* hospitality
* technical
* community
* reliability

These are stable protocol identifiers. Unknown values must be rejected.

Conceptual data models

TrustRelationship:

* schema_version: u8
* subject_agent: AgentPubKey
* context: TrustContext

The author must be derived from the authenticated Holochain action author, not accepted as a user-controlled entry field.

TrustRelationshipRemoval:

* schema_version: u8
* relationship_action: ActionHash

The removal author must match the original relationship author.

Required coordinator API

* create_trust_relationship
* remove_trust_relationship
* get_trust_relationship
* get_trust_relationships_by_author
* get_trust_relationships_for_subject
* get_trust_relationships_by_context
* get_trust_relationship_history

Current approval state

The Trust specification is marked Proposed for Approval. ADR-004 is marked Proposed.

Do not silently mark either document approved or accepted. Record explicit approval before treating the design as formally accepted. Once approval is recorded, update the specification, ADR, and roadmap consistently.

5. Repository Structure and Existing Toolchain

Current Rust workspace members include:

* dnas/holotrust/zomes/identity/integrity
* dnas/holotrust/zomes/identity/coordinator
* dnas/holotrust/zomes/attestation/integrity
* dnas/holotrust/zomes/attestation/coordinator
* dnas/holotrust/zomes/reputation/integrity
* dnas/holotrust/zomes/reputation/coordinator

Workspace:
Cargo.toml

DNA manifest:
dnas/holotrust/workdir/dna.yaml

hApp manifest:
dnas/holotrust/workdir/happ.yaml

Current workspace package settings:

* Rust edition: 2021
* hdi = 0.8.1
* hdk = 0.7.1
* holochain_serialized_bytes = 0.0.57

These versions are pinned in the workspace. Do not change them without a verified compatibility reason.

Existing identity, attestation, and reputation zomes are the implementation references. Inspect their actual code before writing Trust code. Match their established macros, entry definitions, validation callbacks, link patterns, coordinator calls, serialization, and testing conventions.

6. Engineering and Testing Rules

These rules are mandatory because unnecessary long CI runs have previously cost substantial time.

Required validation order

1. Inspect all affected source files and manifests.
2. Check types, imports, visibility, dependencies, paths, zome names, and API compatibility.
3. Check integrity validation logic and authorization boundaries.
4. Inspect test coverage and likely failure points.
5. Run formatting and static checks.
6. Run the smallest relevant targeted tests.
7. Run Fast CI.
8. Run Full CI only after all preliminary checks pass.

Never start an expensive Full CI run while obvious static, formatting, compilation, or focused-test problems remain.

Before asking for a long test run, explain what has already been inspected and why the remaining test is necessary.

Do not claim a test passed unless its actual result has been verified.

Change management

* Prefer small, independently verifiable changes.
* Preserve working code.
* Do not rewrite unrelated modules.
* Do not remove requirements to make tests pass.
* Do not weaken security validation to bypass a failing test.
* Keep documentation aligned with implementation.
* Inspect the complete current file before proposing a replacement.
* When a file needs editing, provide its complete final content rather than scattered fragments.

7. User Collaboration and Delivery Preferences

The project owner primarily works from an iPhone and uses GitHub’s web/mobile editing interface.

Therefore:

* Provide direct GitHub edit links for files that need changes.
* Provide one complete copy-and-paste replacement per file.
* Avoid fragmented code instructions and unnecessary terminal workflows.
* Explain exactly which file is being replaced.
* Do not ask the owner to repeat information already documented here.
* Continue from the latest verified repository state when the owner says “Devam”.
* Do not claim to have committed or pushed changes unless the repository confirms it.
* The owner performs the actual GitHub save/commit when write access is unavailable.

8. Implementation Roadmap

Phase 5 — Trust Graph

1. Obtain explicit approval of HT-TRUST-001 and acceptance of ADR-004.
2. Inspect all existing zome patterns and integration-test infrastructure.
3. Design Trust integrity data structures and validation.
4. Implement trust_integrity.
5. Implement trust coordinator.
6. Implement immutable removal records and authorization.
7. Implement discovery links and queries.
8. Add focused integrity and coordinator tests.
9. Register zomes in Cargo and DNA manifests.
10. Add multi-agent tests with at least three independent agents.
11. Integrate Trust Graph into the real frontend.
12. Run static checks, focused tests, Fast CI, and then Full CI.

Phase 6 — Economic Integration

Investigate and integrate the relevant economic coordination architecture, including:

* hREA
* Unyt
* Holochain economic infrastructure
* HoloFuel-compatible coordination models

Validate protocol compatibility, architecture, dependencies, and actual implementation status before integrating.

Phase 7 — Complete Application and Agent Workflows

Build out the complete application experience:

* Identity and profile discovery.
* Attestation creation and verification.
* Contextual reputation.
* Trust relationship creation and discovery.
* Incoming/outgoing relationship views.
* History and authorization controls.
* Real conductor-backed operations.
* Multi-agent user workflows.

Phase 8 — Economic Application Workflows

Connect agent identity, trust, reputation, and economic coordination into coherent end-to-end workflows. Verify real interactions rather than mock demonstrations.

Phase 9 — Deployment and Network Integration

Prepare deployment and operational infrastructure:

* Reproducible builds.
* Conductor configuration.
* hApp packaging.
* Network deployment.
* Monitoring and observability.
* Backup and recovery considerations.
* Operational documentation.

Phase 10 — Security and Production Readiness

Complete security review, authorization tests, abuse-case analysis, performance checks, compatibility checks, and production deployment verification.

These phases describe the full intended scope. They are not permission to discard or indefinitely defer any capability.

9. Immediate Next Actions

1. Verify that this continuity document has been saved.
2. Update docs/ROADMAP.md to reflect the saved Trust specification and ADR-004 accurately.
3. Keep approval states explicit and consistent.
4. Inspect the existing identity, attestation, and reputation source files individually, including their tests.
5. Inspect the current workflow and integration-test support before implementation.
6. Once the Trust architecture is explicitly approved, implement the Trust integrity layer first.
7. Continue with the coordinator, focused tests, registration, multi-agent tests, and frontend integration.

Do not start by blindly adding files or changing manifests. First understand the existing implementation patterns and test harness.

10. Instructions for Future Work Sessions

At the beginning of a new HoloTrust development session:

1. Read this document.
2. Read docs/ROADMAP.md.
3. Check the latest repository commit and current working tree state.
4. Verify the actual contents and status of the current phase’s specification and ADR.
5. Inspect the latest CI results before deciding what to test.
6. Continue from the last verified completed task.

This document preserves project decisions and handoff information. It is not a copy of every historical conversation or every previous code diff. The repository and its verified commits remain the source of truth.

⸻

HoloTrust — Building the trust infrastructure of the agent-centric internet.
