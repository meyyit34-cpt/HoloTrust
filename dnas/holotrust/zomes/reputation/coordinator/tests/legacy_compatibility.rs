mod support;

use holochain::prelude::{ActionHash, AgentPubKey};
use holochain::sweettest::*;
use reputation_coordinator::{
    ContextualReputationSummary, CreateContextualReviewInput, GetContextualReviewsInput,
};
use reputation_integrity::ReputationDomain;
use std::path::PathBuf;
use support::{ClaimType, CreateAttestationInput, EvidenceReference};

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct LegacyReviewInput {
    subject_agent: AgentPubKey,
    rating: u8,
    comment: Option<String>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct LegacyReviewsQuery {
    subject_agent: AgentPubKey,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct LegacySummary {
    review_count: u32,
    average_rating: f64,
}

#[tokio::test(flavor = "multi_thread")]
async fn legacy_reviews_remain_readable_and_excluded_from_contextual_reputation() {
    let dna_path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../workdir/holotrust.dna");

    assert!(
        dna_path.exists(),
        "Packaged DNA not found at: {}",
        dna_path.display()
    );

    let dna = SweetDnaFile::from_bundle(&dna_path)
        .await
        .expect("Failed to load DNA bundle");

    let mut conductors = SweetConductorBatch::standard(2).await;

    let apps = conductors
        .setup_app("holotrust", [&dna])
        .await
        .expect("Failed to install apps");

    conductors.exchange_peer_info().await;

    let reviewer = apps[0].agent().clone();
    let subject = apps[1].agent().clone();

    let identity_reviewer = apps[0].cells()[0].zome("identity_coordinator");
    let identity_subject = apps[1].cells()[0].zome("identity_coordinator");

    let attestation_reviewer = apps[0].cells()[0].zome("attestation_coordinator");

    let reputation_reviewer = apps[0].cells()[0].zome("reputation_coordinator");
    let reputation_subject = apps[1].cells()[0].zome("reputation_coordinator");

    // Publish a legacy schema-v1 review through Identity.
    let legacy_review: ActionHash = conductors[0]
        .sweet_handle()
        .call(
            &identity_reviewer,
            "create_review",
            LegacyReviewInput {
                subject_agent: subject.clone(),
                rating: 4,
                comment: Some("Legacy review remains available".into()),
            },
        )
        .await;

    // Create valid evidence for the contextual review.
    let evidence: ActionHash = conductors[0]
        .sweet_handle()
        .call(
            &attestation_reviewer,
            "create_attestation",
            CreateAttestationInput {
                subject: subject.clone(),
                claim_type: ClaimType::Skill,
                claim: "Verified contextual reputation evidence".into(),
                evidence: Vec::<EvidenceReference>::new(),
                expires_at: None,
                supersedes: None,
            },
        )
        .await;

    let contextual_review: ActionHash = conductors[0]
        .sweet_handle()
        .call(
            &reputation_reviewer,
            "create_contextual_review",
            CreateContextualReviewInput {
                subject_agent: subject.clone(),
                domain: ReputationDomain::Commerce,
                rating: 5,
                comment: Some("Contextual review remains separate".into()),
                evidence_attestations: vec![evidence],
            },
        )
        .await;

    await_consistency(apps.iter().flat_map(|app| app.cells().iter()))
        .await
        .expect("Legacy and contextual records did not reach DHT consistency");

    // Legacy review must remain readable through the old Identity API.
    let legacy_reviews: Vec<holochain::prelude::Record> = conductors[1]
        .sweet_handle()
        .call(
            &identity_subject,
            "get_reviews",
            LegacyReviewsQuery {
                subject_agent: subject.clone(),
            },
        )
        .await;

    assert_eq!(
        legacy_reviews.len(),
        1,
        "The legacy Identity API must still return the original review"
    );

    assert_eq!(
        legacy_reviews[0].action_address(),
        &legacy_review,
        "The original legacy review must remain unchanged"
    );

    let legacy_summary: LegacySummary = conductors[1]
        .sweet_handle()
        .call(
            &identity_subject,
            "get_reputation_summary",
            LegacyReviewsQuery {
                subject_agent: subject.clone(),
            },
        )
        .await;

    assert_eq!(legacy_summary.review_count, 1);
    assert_eq!(legacy_summary.average_rating, 4.0);

    // Contextual reputation must contain only contextual reviews.
    let contextual_reviews: Vec<holochain::prelude::Record> = conductors[1]
        .sweet_handle()
        .call(
            &reputation_subject,
            "get_contextual_reviews",
            GetContextualReviewsInput {
                subject_agent: subject.clone(),
                domain: None,
            },
        )
        .await;

    assert_eq!(
        contextual_reviews.len(),
        1,
        "Legacy reviews must not appear in contextual review results"
    );

    assert_eq!(
        contextual_reviews[0].action_address(),
        &contextual_review,
        "The contextual result must contain the contextual review"
    );

    let contextual_summaries: Vec<ContextualReputationSummary> = conductors[1]
        .sweet_handle()
        .call(
            &reputation_subject,
            "get_contextual_reputation_summary",
            GetContextualReviewsInput {
                subject_agent: subject,
                domain: None,
            },
        )
        .await;

    let commerce = contextual_summaries
        .iter()
        .find(|summary| summary.domain == ReputationDomain::Commerce)
        .expect("Commerce summary missing");

    assert_eq!(commerce.review_count, 1);
    assert_eq!(commerce.average_rating, 5.0);

    let professional = contextual_summaries
        .iter()
        .find(|summary| summary.domain == ReputationDomain::Professional)
        .expect("Professional summary missing");

    assert_eq!(
        professional.review_count, 0,
        "Legacy reviews must not affect empty contextual domains"
    );

    assert_eq!(professional.average_rating, 0.0);

    // The reviewer identity is deliberately used in both systems;
    // the two records must still remain independently addressable.
    assert_ne!(legacy_review, contextual_review);

    assert_ne!(reviewer, subject);
}
