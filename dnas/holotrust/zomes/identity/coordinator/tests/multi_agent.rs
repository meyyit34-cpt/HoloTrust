use hdk::prelude::*;
use holochain::sweettest::{await_consistency, SweetConductorBatch, SweetDnaFile};
use identity_coordinator::{
    CreateProfileInput, CreateReviewInput, GetProfileInput, GetReviewsInput, ReputationSummary,
    UpdateProfileInput,
};
use identity_integrity::{AgentProfile, AgentType, ReputationReview};
use std::path::PathBuf;

#[tokio::test(flavor = "multi_thread")]
async fn multi_agent_identity_and_reputation_security() {
    // Load the packaged DNA only once.
    let dna_path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../workdir/holotrust.dna");

    assert!(
        dna_path.exists(),
        "Packaged DNA not found: {}",
        dna_path.display()
    );

    let dna_file = SweetDnaFile::from_bundle(&dna_path)
        .await
        .expect("Could not load DNA");

    // Reuse one three-agent network for all scenarios.
    let mut conductors = SweetConductorBatch::standard(3).await;

    let apps = conductors
        .setup_app("holotrust", &[dna_file])
        .await
        .expect("Could not set up apps");

    let alice_cell = apps[0].cells()[0].clone();
    let bob_cell = apps[1].cells()[0].clone();
    let carol_cell = apps[2].cells()[0].clone();

    // 1. Alice creates a profile.
    let alice_profile_hash: ActionHash = conductors[0]
        .call_fallible::<_, ActionHash>(
            &alice_cell.zome("identity_coordinator"),
            "create_profile",
            CreateProfileInput {
                display_name: "Alice".to_string(),
                agent_type: AgentType::Human,
                description: Some("Alice profile".to_string()),
            },
        )
        .await
        .expect("Alice profile creation failed");

    await_consistency(&[alice_cell.clone(), bob_cell.clone(), carol_cell.clone()])
        .await
        .expect("Profile DHT consistency failed");

    // 2. Bob must be able to discover Alice's profile.
    let shared_profile: Option<Record> = conductors[1]
        .call_fallible::<_, Option<Record>>(
            &bob_cell.zome("identity_coordinator"),
            "get_profile",
            GetProfileInput {
                agent_pub_key: alice_cell.agent_pubkey().clone(),
            },
        )
        .await
        .expect("Could not retrieve Alice's profile");

    let shared_profile = shared_profile.expect("Alice's profile was not shared");

    let profile = shared_profile
        .entry()
        .to_app_option::<AgentProfile>()
        .expect("Could not decode Alice's profile")
        .expect("Alice's profile entry is missing");

    assert_eq!(profile.display_name, "Alice");

    // 3. Bob must not be allowed to modify Alice's profile.
    let unauthorized_update = conductors[1]
        .call_fallible::<_, ActionHash>(
            &bob_cell.zome("identity_coordinator"),
            "update_profile",
            UpdateProfileInput {
                action_hash: alice_profile_hash,
                display_name: "Modified by Bob".to_string(),
                agent_type: AgentType::Human,
                description: Some("Unauthorized update".to_string()),
            },
        )
        .await;

    assert!(
        unauthorized_update.is_err(),
        "Bob must not modify Alice's profile"
    );

    // Verify that Alice's original profile remains unchanged.
    let unchanged_profile: Option<Record> = conductors[0]
        .call_fallible::<_, Option<Record>>(
            &alice_cell.zome("identity_coordinator"),
            "get_profile",
            GetProfileInput {
                agent_pub_key: alice_cell.agent_pubkey().clone(),
            },
        )
        .await
        .expect("Could not retrieve Alice's profile after unauthorized update");

    let unchanged_profile =
        unchanged_profile.expect("Alice's profile disappeared after unauthorized update");

    let profile = unchanged_profile
        .entry()
        .to_app_option::<AgentProfile>()
        .expect("Could not decode Alice's profile")
        .expect("Alice's profile entry is missing");

    assert_eq!(
        profile.display_name, "Alice",
        "Unauthorized update changed Alice's profile"
    );

    // 4. Bob and Carol independently review Alice.
    let bob_review_hash: ActionHash = conductors[1]
        .call_fallible::<_, ActionHash>(
            &bob_cell.zome("identity_coordinator"),
            "create_review",
            CreateReviewInput {
                subject_agent: alice_cell.agent_pubkey().clone(),
                rating: 5,
                comment: Some("Reliable agent".to_string()),
            },
        )
        .await
        .expect("Bob's review creation failed");

    let carol_review_hash: ActionHash = conductors[2]
        .call_fallible::<_, ActionHash>(
            &carol_cell.zome("identity_coordinator"),
            "create_review",
            CreateReviewInput {
                subject_agent: alice_cell.agent_pubkey().clone(),
                rating: 4,
                comment: Some("Good collaboration".to_string()),
            },
        )
        .await
        .expect("Carol's review creation failed");

    await_consistency(&[alice_cell.clone(), bob_cell.clone(), carol_cell.clone()])
        .await
        .expect("Review DHT consistency failed");

    // 5. Alice must discover both independent reviews.
    let reviews: Vec<Record> = conductors[0]
        .call_fallible::<_, Vec<Record>>(
            &alice_cell.zome("identity_coordinator"),
            "get_reviews",
            GetReviewsInput {
                subject_agent: alice_cell.agent_pubkey().clone(),
            },
        )
        .await
        .expect("Could not retrieve reviews");

    assert_eq!(reviews.len(), 2, "Expected exactly two independent reviews");

    let review_hashes: Vec<ActionHash> = reviews
        .iter()
        .map(|record| record.action_address().clone())
        .collect();

    assert!(review_hashes.contains(&bob_review_hash));
    assert!(review_hashes.contains(&carol_review_hash));

    for record in &reviews {
        let review = record
            .entry()
            .to_app_option::<ReputationReview>()
            .expect("Could not decode review")
            .expect("Review entry is missing");

        assert_eq!(
            review.subject_agent,
            alice_cell.agent_pubkey().clone(),
            "Review subject does not match Alice"
        );
    }

    // 6. Bob must not be able to submit a duplicate review.
    let duplicate_review = conductors[1]
        .call_fallible::<_, ActionHash>(
            &bob_cell.zome("identity_coordinator"),
            "create_review",
            CreateReviewInput {
                subject_agent: alice_cell.agent_pubkey().clone(),
                rating: 5,
                comment: Some("Duplicate review".to_string()),
            },
        )
        .await;

    assert!(
        duplicate_review.is_err(),
        "Duplicate review should be rejected"
    );

    // 7. Alice must not be able to review herself.
    let self_review = conductors[0]
        .call_fallible::<_, ActionHash>(
            &alice_cell.zome("identity_coordinator"),
            "create_review",
            CreateReviewInput {
                subject_agent: alice_cell.agent_pubkey().clone(),
                rating: 5,
                comment: Some("Self review".to_string()),
            },
        )
        .await;

    assert!(self_review.is_err(), "Self-review must be rejected");

    // 8. Reputation summary must reflect both valid reviews.
    let summary: ReputationSummary = conductors[0]
        .call_fallible::<_, ReputationSummary>(
            &alice_cell.zome("identity_coordinator"),
            "get_reputation_summary",
            GetReviewsInput {
                subject_agent: alice_cell.agent_pubkey().clone(),
            },
        )
        .await
        .expect("Could not retrieve reputation summary");

    assert_eq!(summary.review_count, 2);
    assert!(
        (summary.average_rating - 4.5).abs() < 0.001,
        "Expected average rating 4.5, got {}",
        summary.average_rating
    );
}
