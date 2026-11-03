use hdk::prelude::*;
use holochain::sweettest::{await_consistency, SweetConductorBatch, SweetDnaFile};
use identity_integrity::{AgentProfile, AgentType, ReputationReview};
use std::path::PathBuf;

#[tokio::test(flavor = "multi_thread")]
async fn independent_agents_can_review_same_subject() {
    let dna_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../workdir/holotrust.dna");

    assert!(dna_path.exists());

    let dna_file = SweetDnaFile::from_bundle(&dna_path)
        .await
        .expect("Could not load HoloTrust DNA");

    let mut conductors = SweetConductorBatch::standard(3).await;

    let apps = conductors
        .setup_app("holotrust-review-test", &[dna_file])
        .await
        .expect("Could not install app");

    conductors.exchange_peer_info().await;

    let alice_cell = apps[0].cells()[0].clone();
    let bob_cell = apps[1].cells()[0].clone();
    let carol_cell = apps[2].cells()[0].clone();

    let bob = bob_cell.zome("identity_coordinator");
    let carol = carol_cell.zome("identity_coordinator");
    let alice = alice_cell.zome("identity_coordinator");

    let bob_review: ActionHash = conductors[1]
        .call(
            &bob,
            "create_review",
            identity_coordinator::CreateReviewInput {
                subject_agent: alice_cell.agent_pubkey().clone(),
                rating: 5,
                comment: Some("Reliable agent".into()),
            },
        )
        .await;

    assert!(!bob_review.get_raw_36().is_empty());

    let carol_review: ActionHash = conductors[2]
        .call(
            &carol,
            "create_review",
            identity_coordinator::CreateReviewInput {
                subject_agent: alice_cell.agent_pubkey().clone(),
                rating: 4,
                comment: Some("Helpful collaborator".into()),
            },
        )
        .await;

    assert!(!carol_review.get_raw_36().is_empty());

    await_consistency(&[alice_cell.clone(), bob_cell.clone(), carol_cell.clone()])
        .await
        .expect("DHT consistency failed");

    let reviews: Vec<Record> = conductors[0]
        .call(
            &alice,
            "get_reviews",
            identity_coordinator::GetReviewsInput {
                subject_agent: alice_cell.agent_pubkey().clone(),
            },
        )
        .await;

    assert_eq!(reviews.len(), 2);

    let duplicate = conductors[1]
        .call_fallible::<_, ActionHash>(
            &bob,
            "create_review",
            identity_coordinator::CreateReviewInput {
                subject_agent: alice_cell.agent_pubkey().clone(),
                rating: 3,
                comment: Some("Duplicate attempt".into()),
            },
        )
        .await;

    assert!(
        duplicate.is_err(),
        "SECURITY FAILURE: duplicate review accepted"
    );

    let summary: identity_coordinator::ReputationSummary = conductors[0]
        .call(
            &alice,
            "get_reputation_summary",
            identity_coordinator::GetReviewsInput {
                subject_agent: alice_cell.agent_pubkey().clone(),
            },
        )
        .await;

    assert_eq!(summary.review_count, 2);
    assert!((summary.average_rating - 4.5).abs() < f64::EPSILON);

    await_consistency(&[alice_cell, bob_cell, carol_cell])
        .await
        .expect("Final DHT consistency failed");
}
    );

    let dna_file = SweetDnaFile::from_bundle(&dna_path)
        .await
        .expect("Could not load HoloTrust DNA");

    let mut conductors = SweetConductorBatch::standard(2).await;

    let apps = conductors
        .setup_app("holotrust-test", &[dna_file])
        .await
        .expect("Could not install app");

    conductors.exchange_peer_info().await;

    let alice_cell = apps[0].cells()[0].clone();
    let bob_cell = apps[1].cells()[0].clone();

    let alice = alice_cell.zome("identity_coordinator");
    let bob = bob_cell.zome("identity_coordinator");

    let profile_hash: ActionHash = conductors[0]
        .call(
            &alice,
            "create_profile",
            identity_coordinator::CreateProfileInput {
                display_name: "Alice HoloTrust".into(),
                agent_type: AgentType::Human,
                description: Some("First network agent".into()),
            },
        )
        .await;

    assert!(!profile_hash.get_raw_36().is_empty());

    await_consistency(&[alice_cell.clone(), bob_cell.clone()])
        .await
        .expect("DHT consistency failed");

    let alice_profile: Option<Record> = conductors[1]
        .call(
            &bob,
            "get_profile",
            identity_coordinator::GetProfileInput {
                agent_pub_key: alice_cell.agent_pubkey().clone(),
            },
        )
        .await;

    let alice_profile = alice_profile.expect("Bob could not discover Alice's profile");

    let decoded_profile = alice_profile
        .entry()
        .to_app_option::<AgentProfile>()
        .expect("Could not decode profile")
        .expect("Profile entry missing");

    assert_eq!(decoded_profile.display_name, "Alice HoloTrust");

    let unauthorized_update = conductors[1]
        .call_fallible::<_, ActionHash>(
            &bob,
            "update_profile",
            identity_coordinator::UpdateProfileInput {
                action_hash: alice_profile.action_address().clone(),
                display_name: "Modified by Bob".into(),
                agent_type: AgentType::Human,
                description: Some("Unauthorized modification".into()),
            },
        )
        .await;

    assert!(
        unauthorized_update.is_err(),
        "SECURITY FAILURE: Bob modified Alice's profile"
    );

    // An invalid profile update must be rejected.
    let invalid_profile_update = conductors[0]
        .call_fallible::<_, ActionHash>(
            &alice,
            "update_profile",
            identity_coordinator::UpdateProfileInput {
                action_hash: profile_hash.clone(),
                display_name: "   ".into(),
                agent_type: AgentType::Human,
                description: None,
            },
        )
        .await;

    assert!(
        invalid_profile_update.is_err(),
        "SECURITY FAILURE: an empty profile name was accepted"
    );

    let original_profile: Option<Record> = conductors[0].call(&alice, "get_my_profile", ()).await;

    let original_profile = original_profile.expect("Alice's profile disappeared");

    let original_decoded = original_profile
        .entry()
        .to_app_option::<AgentProfile>()
        .expect("Could not decode original profile")
        .expect("Original profile entry missing");

    assert_eq!(original_decoded.display_name, "Alice HoloTrust");

    // Bob creates a valid review for Alice.
    let review_hash: ActionHash = conductors[1]
        .call(
            &bob,
            "create_review",
            identity_coordinator::CreateReviewInput {
                subject_agent: alice_cell.agent_pubkey().clone(),
                rating: 5,
                comment: Some("Reliable agent".into()),
            },
        )
        .await;

    assert!(!review_hash.get_raw_36().is_empty());

    await_consistency(&[alice_cell.clone(), bob_cell.clone()])
        .await
        .expect("DHT consistency failed after review creation");

    // Alice can retrieve Bob's review.
    let reviews: Vec<Record> = conductors[0]
        .call(
            &alice,
            "get_reviews",
            identity_coordinator::GetReviewsInput {
                subject_agent: alice_cell.agent_pubkey().clone(),
            },
        )
        .await;

    assert_eq!(reviews.len(), 1);

    let decoded_review = reviews[0]
        .entry()
        .to_app_option::<ReputationReview>()
        .expect("Could not decode review")
        .expect("Review entry missing");

    assert_eq!(
        decoded_review.subject_agent,
        alice_cell.agent_pubkey().clone()
    );
    assert_eq!(decoded_review.rating, 5);
    assert_eq!(decoded_review.comment.as_deref(), Some("Reliable agent"));

    // Alice's reputation summary reflects the valid review.
    let summary: identity_coordinator::ReputationSummary = conductors[0]
        .call(
            &alice,
            "get_reputation_summary",
            identity_coordinator::GetReviewsInput {
                subject_agent: alice_cell.agent_pubkey().clone(),
            },
        )
        .await;

    assert_eq!(summary.review_count, 1);
    assert!((summary.average_rating - 5.0).abs() < f64::EPSILON);

    // Bob cannot review Alice more than once.
    let duplicate_review = conductors[1]
        .call_fallible::<_, ActionHash>(
            &bob,
            "create_review",
            identity_coordinator::CreateReviewInput {
                subject_agent: alice_cell.agent_pubkey().clone(),
                rating: 4,
                comment: Some("Duplicate review".into()),
            },
        )
        .await;

    assert!(
        duplicate_review.is_err(),
        "SECURITY FAILURE: Bob created a duplicate review"
    );

    // Bob cannot review himself.
    let self_review = conductors[1]
        .call_fallible::<_, ActionHash>(
            &bob,
            "create_review",
            identity_coordinator::CreateReviewInput {
                subject_agent: bob_cell.agent_pubkey().clone(),
                rating: 5,
                comment: Some("Self review".into()),
            },
        )
        .await;

    assert!(
        self_review.is_err(),
        "SECURITY FAILURE: Bob reviewed himself"
    );

    // Ratings below 1 and above 5 must be rejected.
    for invalid_rating in [0, 6] {
        let invalid_review = conductors[0]
            .call_fallible::<_, ActionHash>(
                &alice,
                "create_review",
                identity_coordinator::CreateReviewInput {
                    subject_agent: bob_cell.agent_pubkey().clone(),
                    rating: invalid_rating,
                    comment: Some("Invalid rating test".into()),
                },
            )
            .await;

        assert!(
            invalid_review.is_err(),
            "SECURITY FAILURE: rating {} was accepted",
            invalid_rating
        );
    }

    // Control characters in review comments must be rejected.
    let invalid_comment = conductors[0]
        .call_fallible::<_, ActionHash>(
            &alice,
            "create_review",
            identity_coordinator::CreateReviewInput {
                subject_agent: bob_cell.agent_pubkey().clone(),
                rating: 4,
                comment: Some("Invalid\ncomment".into()),
            },
        )
        .await;

    assert!(
        invalid_comment.is_err(),
        "SECURITY FAILURE: a review with control characters was accepted"
    );

    await_consistency(&[alice_cell, bob_cell])
        .await
        .expect("DHT consistency failed after reputation tests");
}
#[tokio::test(flavor = "multi_thread")]
async fn independent_agents_can_review_same_subject() {
    let dna_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../workdir/holotrust.dna");

    assert!(dna_path.exists());

    let dna_file = SweetDnaFile::from_bundle(&dna_path)
        .await
        .expect("Could not load HoloTrust DNA");

    let mut conductors = SweetConductorBatch::standard(3).await;

    let apps = conductors
        .setup_app("holotrust-review-test", &[dna_file])
        .await
        .expect("Could not install app");

    conductors.exchange_peer_info().await;

    let alice_cell = apps[0].cells()[0].clone();
    let bob_cell = apps[1].cells()[0].clone();
    let carol_cell = apps[2].cells()[0].clone();

    let bob = bob_cell.zome("identity_coordinator");
    let carol = carol_cell.zome("identity_coordinator");
    let alice = alice_cell.zome("identity_coordinator");

    let bob_review: ActionHash = conductors[1]
        .call(
            &bob,
            "create_review",
            identity_coordinator::CreateReviewInput {
                subject_agent: alice_cell.agent_pubkey().clone(),
                rating: 5,
                comment: Some("Reliable agent".into()),
            },
        )
        .await;

    assert!(!bob_review.get_raw_36().is_empty());

    let carol_review: ActionHash = conductors[2]
        .call(
            &carol,
            "create_review",
            identity_coordinator::CreateReviewInput {
                subject_agent: alice_cell.agent_pubkey().clone(),
                rating: 4,
                comment: Some("Helpful collaborator".into()),
            },
        )
        .await;

    assert!(!carol_review.get_raw_36().is_empty());

    await_consistency(&[
        alice_cell.clone(),
        bob_cell.clone(),
        carol_cell.clone(),
    ])
    .await
    .expect("DHT consistency failed");

    let reviews: Vec<Record> = conductors[0]
        .call(
            &alice,
            "get_reviews",
            identity_coordinator::GetReviewsInput {
                subject_agent: alice_cell.agent_pubkey().clone(),
            },
        )
        .await;

    assert_eq!(reviews.len(), 2);

    let duplicate = conductors[1]
        .call_fallible::<_, ActionHash>(
            &bob,
            "create_review",
            identity_coordinator::CreateReviewInput {
                subject_agent: alice_cell.agent_pubkey().clone(),
                rating: 3,
                comment: Some("Duplicate attempt".into()),
            },
        )
        .await;

    assert!(
        duplicate.is_err(),
        "SECURITY FAILURE: duplicate review accepted"
    );

    let summary: identity_coordinator::ReputationSummary =
        conductors[0]
            .call(
                &alice,
                "get_reputation_summary",
                identity_coordinator::GetReviewsInput {
                    subject_agent: alice_cell.agent_pubkey().clone(),
                },
            )
            .await;

    assert_eq!(summary.review_count, 2);
    assert!((summary.average_rating - 4.5).abs() < f64::EPSILON);

    await_consistency(&[
        alice_cell,
        bob_cell,
        carol_cell,
    ])
    .await
    .expect("Final DHT consistency failed");
}
