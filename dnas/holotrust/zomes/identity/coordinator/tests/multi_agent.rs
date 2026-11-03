use std::path::PathBuf;

use hdk::prelude::*;
use holochain::sweettest::*;

#[tokio::test(flavor = "multi_thread")]
async fn multi_agent_identity_and_reputation_security() {
    // Load the packaged DNA only once.
    let dna_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../workdir/holotrust.dna");

    assert!(
        dna_path.exists(),
        "Packaged DNA not found at: {}",
        dna_path.display()
    );

    let dna = DnaFile::from_bundle(&dna_path)
        .await
        .expect("Failed to load DNA bundle");

    let mut conductors = SweetConductorBatch::standard(3).await;
    let apps = conductors
        .setup_app("holotrust", [&dna])
        .await
        .expect("Failed to install apps");

    let alice = apps[0].agent_pubkey().clone();
    let bob = apps[1].agent_pubkey().clone();
    let carol = apps[2].agent_pubkey().clone();

    // Alice creates her identity.
    let alice_profile = conductors[0]
        .call(&apps[0], "identity", "create_profile", ())
        .await;

    // Bob can discover Alice's profile.
    let discovered_profile = conductors[1]
        .call(&apps[1], "identity", "get_profile", alice.clone())
        .await;

    assert_eq!(discovered_profile, Some(alice_profile.clone()));

    // Bob cannot update Alice's profile.
    let unauthorized_update = conductors[1]
        .call_fallible(
            &apps[1],
            "identity",
            "update_profile",
            alice_profile.clone(),
        )
        .await;

    assert!(unauthorized_update.is_err());

    // Bob and Carol create reviews for Alice.
    let bob_review = conductors[1]
        .call(&apps[1], "reputation", "create_review", ())
        .await;

    let carol_review = conductors[2]
        .call(&apps[2], "reputation", "create_review", ())
        .await;

    // Alice can retrieve both reviews.
    let reviews = conductors[0]
        .call(&apps[0], "reputation", "get_reviews", alice.clone())
        .await;

    assert_eq!(reviews.len(), 2);
    assert!(reviews.iter().any(|review| review == &bob_review));
    assert!(reviews.iter().any(|review| review == &carol_review));

    // Duplicate reviews are rejected.
    let duplicate_review = conductors[1]
        .call_fallible(&apps[1], "reputation", "create_review", ())
        .await;

    assert!(duplicate_review.is_err());

    // Self-reviews are rejected.
    let self_review = conductors[0]
        .call_fallible(&apps[0], "reputation", "create_review", ())
        .await;

    assert!(self_review.is_err());

    // Verify the reputation summary.
    let summary = conductors[0]
        .call(&apps[0], "reputation", "get_reputation_summary", alice)
        .await;

    assert_eq!(summary.review_count, 2);
    assert_eq!(summary.average_rating, 4.5);
}
