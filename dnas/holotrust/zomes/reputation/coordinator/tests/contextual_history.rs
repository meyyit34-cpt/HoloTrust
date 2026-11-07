mod support;

use holochain::prelude::{ActionHash, Record};
use holochain::sweettest::*;
use reputation_coordinator::{
    ContextualReputationSummary, CreateContextualReviewInput, GetContextualReviewsInput,
};
use reputation_integrity::ReputationDomain;
use std::path::PathBuf;
use support::{
    AttestationStatus, ClaimType, CreateAttestationInput, EvidenceReference,
    GetAttestationInput,
};};
use reputation_integrity::ReputationDomain;
use std::path::PathBuf;

#[tokio::test(flavor = "multi_thread")]
async fn superseded_evidence_preserves_history_but_not_current_reputation() {
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

    let subject = apps[2].agent().clone();

    let attestation_a = apps[0].cells()[0].zome("attestation_coordinator");
    let reputation_a = apps[0].cells()[0].zome("reputation_coordinator");
    let reputation_b = apps[1].cells()[0].zome("reputation_coordinator");
    let reputation_subject = apps[2].cells()[0].zome("reputation_coordinator");

    let original_evidence: ActionHash = conductors[0]
        .sweet_handle()
        .call(
            &attestation_a,
            "create_attestation",
            CreateAttestationInput {
                subject: subject.clone(),
                claim_type: ClaimType::Skill,
                claim: "Original verified skill".into(),
                evidence: Vec::<EvidenceReference>::new(),
                expires_at: None,
                supersedes: None,
            },
        )
        .await;

    await_consistency(apps.iter().flat_map(|app| app.cells().iter()))
        .await
        .expect("Original evidence did not reach consistency");

    let original_review: ActionHash = conductors[0]
        .sweet_handle()
        .call(
            &reputation_a,
            "create_contextual_review",
            CreateContextualReviewInput {
                subject_agent: subject.clone(),
                domain: ReputationDomain::Commerce,
                rating: 5,
                comment: Some("Review based on original evidence".into()),
                evidence_attestations: vec![original_evidence.clone()],
            },
        )
        .await;

    await_consistency(apps.iter().flat_map(|app| app.cells().iter()))
        .await
        .expect("Original review did not reach consistency");

    let replacement_evidence: ActionHash = conductors[0]
        .sweet_handle()
        .call(
            &attestation_a,
            "create_attestation",
            CreateAttestationInput {
                subject: subject.clone(),
                claim_type: ClaimType::Skill,
                claim: "Updated verified skill".into(),
                evidence: Vec::<EvidenceReference>::new(),
                expires_at: None,
                supersedes: Some(original_evidence.clone()),
            },
        )
        .await;

    await_consistency(apps.iter().flat_map(|app| app.cells().iter()))
        .await
        .expect("Replacement evidence did not reach consistency");

    let status: AttestationStatus = conductors[0]
        .sweet_handle()
        .call(
            &attestation_a,
            "resolve_attestation_status",
            GetAttestationInput {
                action_hash: original_evidence.clone(),
            },
        )
        .await;

    assert_eq!(
        status,
        AttestationStatus::Superseded,
        "Original evidence must be marked as superseded"
    );

    let rejected_review: Result<ActionHash, _> = conductors[1]
        .sweet_handle()
        .call_fallible(
            &reputation_b,
            "create_contextual_review",
            CreateContextualReviewInput {
                subject_agent: subject.clone(),
                domain: ReputationDomain::Reliability,
                rating: 5,
                comment: Some("Must reject superseded evidence".into()),
                evidence_attestations: vec![original_evidence.clone()],
            },
        )
        .await;

    assert!(
        rejected_review.is_err(),
        "New reviews must not accept superseded evidence"
    );

    let historical_reviews: Vec<Record> = conductors[2]
        .sweet_handle()
        .call(
            &reputation_subject,
            "get_contextual_reviews",
            GetContextualReviewsInput {
                subject_agent: subject.clone(),
                domain: Some(ReputationDomain::Commerce),
            },
        )
        .await;

    assert_eq!(
        historical_reviews.len(),
        1,
        "Historical review must remain visible after evidence supersession"
    );

    assert_eq!(
        historical_reviews[0].action_address(),
        &original_review,
        "The original review record must remain unchanged"
    );

    let summaries: Vec<ContextualReputationSummary> = conductors[2]
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

    let commerce = summaries
        .iter()
        .find(|summary| summary.domain == ReputationDomain::Commerce)
        .expect("Commerce summary missing");

    assert_eq!(
        commerce.review_count, 0,
        "Superseded evidence must not contribute to current reputation"
    );

    assert_eq!(
        commerce.average_rating, 0.0,
        "Superseded evidence must not contribute to current average"
    );

    let replacement_status: AttestationStatus = conductors[0]
        .sweet_handle()
        .call(
            &attestation_a,
            "resolve_attestation_status",
            GetAttestationInput {
                action_hash: replacement_evidence,
            },
        )
        .await;

    assert_eq!(
        replacement_status,
        AttestationStatus::Active,
        "Replacement evidence must remain active"
    );
}
