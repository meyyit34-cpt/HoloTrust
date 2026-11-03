use holochain::prelude::{ActionHash, Record};
use holochain::sweettest::*;
use identity_coordinator::{
    CreateProfileInput, CreateReviewInput, GetProfileInput, GetReviewsInput, ReputationSummary,
    UpdateProfileInput,
};
use identity_integrity::AgentType;
use std::path::PathBuf;

#[tokio::test(flavor = "multi_thread")]
async fn multi_agent_identity_and_reputation_security() {
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
    let carol = apps[2].agent().clone();

    let alice_zome = apps[0].cells()[0].zome("identity_coordinator");
    let bob_zome = apps[1].cells()[0].zome("identity_coordinator");
    let carol_zome = apps[2].cells()[0].zome("identity_coordinator");

    let alice_profile: ActionHash = conductors[0]
        .sweet_handle()
        .call(
            &alice_zome,
            "create_profile",
            CreateProfileInput {
                display_name: "Alice".into(),
                agent_type: AgentType::Human,
                description: Some("Alice identity profile".into()),
            },
        )
        .await;

    await_consistency(apps.iter().flat_map(|app| app.cells().iter()))
        .await
        .expect("Alice profile did not reach DHT consistency");

    let discovered_profile: Option<Record> = conductors[1]
        .sweet_handle()
        .call(
            &bob_zome,
            "get_profile",
            GetProfileInput {
                agent_pub_key: alice.clone(),
            },
        )
        .await;

    assert!(discovered_profile.is_some());
    assert_eq!(discovered_profile.unwrap().action().author(), &alice);

    let unauthorized_update: Result<ActionHash, _> = conductors[1]
        .sweet_handle()
        .call_fallible(
            &bob_zome,
            "update_profile",
            UpdateProfileInput {
                action_hash: alice_profile.clone(),
                display_name: "Unauthorized change".into(),
                agent_type: AgentType::Human,
                description: None,
            },
        )
        .await;

    assert!(unauthorized_update.is_err());

    let bob_review_input = CreateReviewInput {
        subject_agent: alice.clone(),
        rating: 5,
        comment: Some("Good experience".into()),
    };

    conductors[1]
        .sweet_handle()
        .call(&bob_zome, "create_review", bob_review_input)
        .await;

    conductors[2]
        .sweet_handle()
        .call(
            &carol_zome,
            "create_review",
            CreateReviewInput {
                subject_agent: alice.clone(),
                rating: 4,
                comment: Some("Reliable agent".into()),
            },
        )
        .await;

    await_consistency(apps.iter().flat_map(|app| app.cells().iter()))
        .await
        .expect("Review records did not reach DHT consistency");

    let reviews: Vec<Record> = conductors[0]
        .sweet_handle()
        .call(
            &alice_zome,
            "get_reviews",
            GetReviewsInput {
                subject_agent: alice.clone(),
            },
        )
        .await;

    assert_eq!(reviews.len(), 2);

    assert!(reviews.iter().any(|review| review.action().author() == &bob));

    assert!(reviews.iter().any(|review| review.action().author() == &carol));

    let duplicate_review: Result<ActionHash, _> = conductors[1]
        .sweet_handle()
        .call_fallible(
            &bob_zome,
            "create_review",
            CreateReviewInput {
                subject_agent: alice.clone(),
                rating: 5,
                comment: Some("Good experience".into()),
            },
        )
        .await;

    assert!(duplicate_review.is_err());

    let self_review: Result<ActionHash, _> = conductors[0]
        .sweet_handle()
        .call_fallible(
            &alice_zome,
            "create_review",
            CreateReviewInput {
                subject_agent: alice.clone(),
                rating: 5,
                comment: Some("Self review".into()),
            },
        )
        .await;

    assert!(self_review.is_err());

    let summary: ReputationSummary = conductors[0]
        .sweet_handle()
        .call(
            &alice_zome,
            "get_reputation_summary",
            GetReviewsInput {
                subject_agent: alice,
            },
        )
        .await;

    assert_eq!(summary.review_count, 2);
    assert_eq!(summary.average_rating, 4.5);
}
