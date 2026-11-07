use holochain::prelude::ActionHash;
use holochain::sweettest::*;
use std::path::PathBuf;
use trust_coordinator::{
    AreTrustRelationshipsMutualInput, CreateTrustRelationshipInput,
    GetTrustRelationshipHistoryInput, GetTrustRelationshipsByAuthorInput,
    GetTrustRelationshipsByContextInput, GetTrustRelationshipsForSubjectInput,
    RemoveTrustRelationshipInput, TrustRelationshipHistory,
};
use trust_integrity::TrustContext;

#[tokio::test(flavor = "multi_thread")]
async fn trust_graph_enforces_direction_context_authorization_and_history() {
    let dna_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../workdir/holotrust.dna");

    assert!(
        dna_path.exists(),
        "Packaged DNA not found at: {}",
        dna_path.display()
    );

    let dna = SweetDnaFile::from_bundle(&dna_path)
        .await
        .expect("Failed to load DNA bundle");

    let mut conductors = SweetConductorBatch::standard(3).await;

    let apps = conductors
        .setup_app("holotrust", [&dna])
        .await
        .expect("Failed to install apps");

    conductors.exchange_peer_info().await;

    let alice = apps[0].agent().clone();
    let bob = apps[1].agent().clone();

    let alice_zome = apps[0].cells()[0].zome("trust_coordinator");
    let bob_zome = apps[1].cells()[0].zome("trust_coordinator");
    let carol_zome = apps[2].cells()[0].zome("trust_coordinator");

    // Alice declares trust toward Bob in the commerce context.
    let alice_to_bob: ActionHash = conductors[0]
        .sweet_handle()
        .call(
            &alice_zome,
            "create_trust_relationship",
            CreateTrustRelationshipInput {
                subject_agent: bob.clone(),
                context: TrustContext::Commerce,
            },
        )
        .await;

    await_consistency(apps.iter().flat_map(|app| app.cells().iter()))
        .await
        .expect("Alice's trust relationship did not reach DHT consistency");

    // A third agent must discover Alice's outgoing relationship.
    let alice_outgoing: Vec<TrustRelationshipHistory> = conductors[2]
        .sweet_handle()
        .call(
            &carol_zome,
            "get_trust_relationships_by_author",
            GetTrustRelationshipsByAuthorInput {
                author: alice.clone(),
            },
        )
        .await;

    assert_eq!(alice_outgoing.len(), 1);
    assert_eq!(
        alice_outgoing[0].relationship.action_address(),
        &alice_to_bob
    );
    assert!(alice_outgoing[0].is_active);

    // A third agent must discover relationships directed toward Bob.
    let bob_incoming: Vec<TrustRelationshipHistory> = conductors[2]
        .sweet_handle()
        .call(
            &carol_zome,
            "get_trust_relationships_for_subject",
            GetTrustRelationshipsForSubjectInput {
                subject_agent: bob.clone(),
            },
        )
        .await;

    assert_eq!(bob_incoming.len(), 1);
    assert_eq!(bob_incoming[0].relationship.action_address(), &alice_to_bob);

    // Context filtering must preserve the requested author scope.
    let commerce_relationships: Vec<TrustRelationshipHistory> = conductors[2]
        .sweet_handle()
        .call(
            &carol_zome,
            "get_trust_relationships_by_context",
            GetTrustRelationshipsByContextInput {
                author: alice.clone(),
                context: TrustContext::Commerce,
            },
        )
        .await;

    assert_eq!(commerce_relationships.len(), 1);

    // One-way trust must not be reported as mutual.
    let initially_mutual: bool = conductors[2]
        .sweet_handle()
        .call(
            &carol_zome,
            "are_trust_relationships_mutual",
            AreTrustRelationshipsMutualInput {
                agent_a: alice.clone(),
                agent_b: bob.clone(),
                context: TrustContext::Commerce,
            },
        )
        .await;

    assert!(!initially_mutual);

    // Bob independently declares trust toward Alice in the same context.
    let _bob_to_alice: ActionHash = conductors[1]
        .sweet_handle()
        .call(
            &bob_zome,
            "create_trust_relationship",
            CreateTrustRelationshipInput {
                subject_agent: alice.clone(),
                context: TrustContext::Commerce,
            },
        )
        .await;

    await_consistency(apps.iter().flat_map(|app| app.cells().iter()))
        .await
        .expect("Mutual trust relationships did not reach DHT consistency");

    let mutual_commerce: bool = conductors[2]
        .sweet_handle()
        .call(
            &carol_zome,
            "are_trust_relationships_mutual",
            AreTrustRelationshipsMutualInput {
                agent_a: alice.clone(),
                agent_b: bob.clone(),
                context: TrustContext::Commerce,
            },
        )
        .await;

    assert!(mutual_commerce);

    // A relationship in another context must not imply mutual trust there.
    let _bob_professional: ActionHash = conductors[1]
        .sweet_handle()
        .call(
            &bob_zome,
            "create_trust_relationship",
            CreateTrustRelationshipInput {
                subject_agent: alice.clone(),
                context: TrustContext::Professional,
            },
        )
        .await;

    await_consistency(apps.iter().flat_map(|app| app.cells().iter()))
        .await
        .expect("Professional relationship did not reach DHT consistency");

    let mutual_professional: bool = conductors[2]
        .sweet_handle()
        .call(
            &carol_zome,
            "are_trust_relationships_mutual",
            AreTrustRelationshipsMutualInput {
                agent_a: alice.clone(),
                agent_b: bob.clone(),
                context: TrustContext::Professional,
            },
        )
        .await;

    assert!(!mutual_professional);

    // Duplicate active relationships in the same context must be rejected.
    let duplicate: Result<ActionHash, _> = conductors[0]
        .sweet_handle()
        .call_fallible(
            &alice_zome,
            "create_trust_relationship",
            CreateTrustRelationshipInput {
                subject_agent: bob.clone(),
                context: TrustContext::Commerce,
            },
        )
        .await;

    assert!(
        duplicate.is_err(),
        "Duplicate active trust relationship was accepted"
    );

    // Self-directed trust must be rejected.
    let self_trust: Result<ActionHash, _> = conductors[0]
        .sweet_handle()
        .call_fallible(
            &alice_zome,
            "create_trust_relationship",
            CreateTrustRelationshipInput {
                subject_agent: alice.clone(),
                context: TrustContext::General,
            },
        )
        .await;

    assert!(
        self_trust.is_err(),
        "Self-directed trust relationship was accepted"
    );

    // Bob must not be allowed to remove Alice's relationship.
    let unauthorized_removal: Result<ActionHash, _> = conductors[1]
        .sweet_handle()
        .call_fallible(
            &bob_zome,
            "remove_trust_relationship",
            RemoveTrustRelationshipInput {
                relationship_action: alice_to_bob.clone(),
            },
        )
        .await;

    assert!(
        unauthorized_removal.is_err(),
        "A different agent removed Alice's relationship"
    );

    // Alice can remove her own relationship.
    let removal_action: ActionHash = conductors[0]
        .sweet_handle()
        .call(
            &alice_zome,
            "remove_trust_relationship",
            RemoveTrustRelationshipInput {
                relationship_action: alice_to_bob.clone(),
            },
        )
        .await;

    assert_ne!(removal_action, alice_to_bob);

    await_consistency(apps.iter().flat_map(|app| app.cells().iter()))
        .await
        .expect("Trust removal did not reach DHT consistency");

    // Historical retrieval must preserve the original record and expose removal.
    let history: Option<TrustRelationshipHistory> = conductors[2]
        .sweet_handle()
        .call(
            &carol_zome,
            "get_trust_relationship_history",
            GetTrustRelationshipHistoryInput {
                relationship_action: alice_to_bob.clone(),
            },
        )
        .await;

    let history = history.expect("Removed relationship history was not found");

    assert_eq!(history.relationship.action_address(), &alice_to_bob);
    assert!(!history.is_active);
    assert!(history.removal.is_some());

    // A second removal must be rejected.
    let duplicate_removal: Result<ActionHash, _> = conductors[0]
        .sweet_handle()
        .call_fallible(
            &alice_zome,
            "remove_trust_relationship",
            RemoveTrustRelationshipInput {
                relationship_action: alice_to_bob.clone(),
            },
        )
        .await;

    assert!(
        duplicate_removal.is_err(),
        "Duplicate trust removal was accepted"
    );

    // A new declaration after removal must be allowed as a new historical record.
    let recreated_relationship: ActionHash = conductors[0]
        .sweet_handle()
        .call(
            &alice_zome,
            "create_trust_relationship",
            CreateTrustRelationshipInput {
                subject_agent: bob.clone(),
                context: TrustContext::Commerce,
            },
        )
        .await;

    assert_ne!(recreated_relationship, alice_to_bob);

    await_consistency(apps.iter().flat_map(|app| app.cells().iter()))
        .await
        .expect("Recreated trust relationship did not reach DHT consistency");

    let final_outgoing: Vec<TrustRelationshipHistory> = conductors[2]
        .sweet_handle()
        .call(
            &carol_zome,
            "get_trust_relationships_by_author",
            GetTrustRelationshipsByAuthorInput {
                author: alice.clone(),
            },
        )
        .await;

    assert_eq!(
        final_outgoing.len(),
        2,
        "Both original and recreated declarations must remain discoverable"
    );

    let active_count = final_outgoing.iter().filter(|item| item.is_active).count();

    let removed_count = final_outgoing.iter().filter(|item| !item.is_active).count();

    assert_eq!(active_count, 1);
    assert_eq!(removed_count, 1);

    let final_commerce: Vec<TrustRelationshipHistory> = conductors[2]
        .sweet_handle()
        .call(
            &carol_zome,
            "get_trust_relationships_by_context",
            GetTrustRelationshipsByContextInput {
                author: alice,
                context: TrustContext::Commerce,
            },
        )
        .await;

    assert_eq!(
        final_commerce.len(),
        2,
        "Context discovery must preserve both active and removed history"
    );

    assert!(
        final_commerce.iter().any(|item| item.is_active),
        "The recreated relationship must be active"
    );

    assert!(
        final_commerce.iter().any(|item| !item.is_active),
        "The original relationship must remain in history as removed"
    );
}
