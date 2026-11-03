use hdk::prelude::*;
use holochain::sweettest::{await_consistency, SweetConductorBatch, SweetDnaFile};
use identity_integrity::{AgentProfile, AgentType, ReputationReview};
use std::path::PathBuf;

#[tokio::test(flavor = "multi_thread")]
async fn two_agents_share_profiles_and_cannot_modify_each_others_data() {
    let dna_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../workdir/holotrust.dna");

    assert!(
        dna_path.exists(),
        "Packaged DNA not found: {}",
        dna_path.display()
    );

    let dna_file = SweetDnaFile::from_bundle(&dna_path)
        .await
        .expect("Could not load DNA");

    let mut conductors = SweetConductorBatch::from_standard_config(2).await;
    let apps = conductors
        .setup_app("holotrust", &[dna_file.clone(), dna_file])
        .await
        .expect("Could not set up apps");

    let alice_cell = apps[0].cells()[0].clone();
    let bob_cell = apps[1].cells()[0].clone();

    let alice_profile = AgentProfile {
        nickname: "Alice".to_string(),
        agent_type: AgentType::Human,
        bio: Some("Alice profile".to_string()),
        avatar: None,
    };

    let alice_profile_hash: ActionHash = conductors[0]
        .call(&alice_cell.zome("identity"), "create_profile", alice_profile)
        .await
        .expect("Alice profile creation failed");

    await_consistency(&[alice_cell.clone(), bob_cell.clone()])
        .await
        .expect("DHT consistency failed");

    let shared_profile: Option<Record> = conductors[1]
        .call(
            &bob_cell.zome("identity"),
            "get_profile",
            alice_cell.agent_pubkey().clone(),
        )
        .await
        .expect("Could not retrieve Alice profile");

    assert!(shared_profile.is_some(), "Alice profile not shared");

    let bob_profile = AgentProfile {
        nickname: "Bob".to_string(),
        agent_type: AgentType::Human,
        bio: Some("Bob profile".to_string()),
        avatar: None,
    };

    let unauthorized_update = conductors[1]
        .call(
            &bob_cell.zome("identity"),
            "update_profile",
            (alice_profile_hash.clone(), bob_profile),
        )
        .await;

    assert!(
        unauthorized_update.is_err(),
        "Bob must not modify Alice's profile"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn independent_agents_can_review_same_subject() {
    let dna_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../workdir/holotrust.dna");

    assert!(
        dna_path.exists(),
        "Packaged DNA not found: {}",
        dna_path.display()
    );

    let dna_file = SweetDnaFile::from_bundle(&dna_path)
        .await
        .expect("Could not load DNA");

    let mut conductors = SweetConductorBatch::from_standard_config(3).await;
    let apps = conductors
        .setup_app(
            "holotrust",
            &[dna_file.clone(), dna_file.clone(), dna_file],
        )
        .await
        .expect("Could not set up apps");

    let alice_cell = apps[0].cells()[0].clone();
    let bob_cell = apps[1].cells()[0].clone();
    let carol_cell = apps[2].cells()[0].clone();

    let bob_review = ReputationReview {
        subject_agent: alice_cell.agent_pubkey().clone(),
        rating: 5,
        comment: Some("Reliable agent".to_string()),
    };

    let bob_review_hash: ActionHash = conductors[1]
        .call(&bob_cell.zome("identity"), "create_review", bob_review.clone())
        .await
        .expect("Bob review creation failed");

    let carol_review = ReputationReview {
        subject_agent: alice_cell.agent_pubkey().clone(),
        rating: 4,
        comment: Some("Good collaboration".to_string()),
    };

    let carol_review_hash: ActionHash = conductors[2]
        .call(
            &carol_cell.zome("identity"),
            "create_review",
            carol_review,
        )
        .await
        .expect("Carol review creation failed");

    await_consistency(&[
        alice_cell.clone(),
        bob_cell.clone(),
        carol_cell.clone(),
    ])
    .await
    .expect("DHT consistency failed");

    let bob_record: Option<Record> = conductors[0]
        .call(
            &alice_cell.zome("identity"),
            "get_review",
            bob_review_hash.clone(),
        )
        .await
        .expect("Could not retrieve Bob review");

    let carol_record: Option<Record> = conductors[0]
        .call(
            &alice_cell.zome("identity"),
            "get_review",
            carol_review_hash.clone(),
        )
        .await
        .expect("Could not retrieve Carol review");

    assert!(bob_record.is_some(), "Bob review missing");
    assert!(carol_record.is_some(), "Carol review missing");

    let duplicate_review = conductors[1]
        .call(
            &bob_cell.zome("identity"),
            "create_review",
            bob_review,
        )
        .await;

    assert!(
        duplicate_review.is_err(),
        "Duplicate review should be rejected"
    );

    let summary: Record = conductors[0]
        .call(
            &alice_cell.zome("identity"),
            "get_reputation_summary",
            alice_cell.agent_pubkey().clone(),
        )
        .await
        .expect("Could not retrieve reputation summary");

    assert!(summary.entry().to_app_option::<serde_json::Value>().unwrap().is_some());
}
