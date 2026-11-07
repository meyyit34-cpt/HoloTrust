mod support;

use holochain::prelude::{ActionHash, Record};
use holochain::sweettest::*;
use reputation_coordinator::{
    ContextualReputationSummary, CreateContextualReviewInput, GetContextualReviewsInput,
};
use reputation_integrity::ReputationDomain;
use std::path::PathBuf;
use support::{
    ClaimType, CreateAttestationInput, EvidenceReference, RevokeAttestationInput,
};
use reputation_integrity::ReputationDomain;
use std::path::PathBuf;

#[tokio::test(flavor = "multi_thread")]
async fn contextual_reputation_enforces_domains_evidence_and_multi_agent_rules() {
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

    let reputation_a = apps[0].cells()[0].zome("reputation_coordinator");
    let reputation_b = apps[1].cells()[0].zome("reputation_coordinator");
    let reputation_subject = apps[2].cells()[0].zome("reputation_coordinator");

    let attestation_a = apps[0].cells()[0].zome("attestation_coordinator");
    let attestation_b = apps[1].cells()[0].zome("attestation_coordinator");

    let evidence: ActionHash = conductors[0]
        .sweet_handle()
        .call(
            &attestation_a,
            "create_attestation",
            CreateAttestationInput {
                subject: subject.clone(),
                claim_type: ClaimType::Skill,
                claim: "Verified technical work".into(),
                evidence: Vec::<EvidenceReference>::new(),
                expires_at: None,
                supersedes: None,
            },
        )
        .await;

    await_consistency(apps.iter().flat_map(|app| app.cells().iter()))
        .await
        .expect("Evidence did not reach DHT consistency");

    let commerce_review: ActionHash = conductors[0]
        .sweet_handle()
        .call(
            &reputation_a,
            "create_contextual_review",
            CreateContextualReviewInput {
                subject_agent: subject.clone(),
                domain: ReputationDomain::Commerce,
                rating: 5,
                comment: Some("Successful transaction".into()),
                evidence_attestations: vec![evidence.clone()],
            },
        )
        .await;

    let duplicate_review: Result<ActionHash, _> = conductors[0]
        .sweet_handle()
        .call_fallible(
            &reputation_a,
            "create_contextual_review",
            CreateContextualReviewInput {
                subject_agent: subject.clone(),
                domain: ReputationDomain::Commerce,
                rating: 4,
                comment: Some("Duplicate review".into()),
                evidence_attestations: vec![evidence.clone()],
            },
        )
        .await;

    assert!(
        duplicate_review.is_err(),
        "The same author must not review the same subject twice in one domain"
    );

    let professional_review: ActionHash = conductors[0]
        .sweet_handle()
        .call(
            &reputation_a,
            "create_contextual_review",
            CreateContextualReviewInput {
                subject_agent: subject.clone(),
                domain: ReputationDomain::Professional,
                rating: 4,
                comment: Some("Professional collaboration".into()),
                evidence_attestations: vec![evidence.clone()],
            },
        )
        .await;

    assert_ne!(professional_review, commerce_review);

    let second_reviewer_review: ActionHash = conductors[1]
        .sweet_handle()
        .call(
            &reputation_b,
            "create_contextual_review",
            CreateContextualReviewInput {
                subject_agent: subject.clone(),
                domain: ReputationDomain::Commerce,
                rating: 3,
                comment: Some("Independent reviewer".into()),
                evidence_attestations: vec![evidence.clone()],
            },
        )
        .await;

    assert_ne!(second_reviewer_review, commerce_review);

    await_consistency(apps.iter().flat_map(|app| app.cells().iter()))
        .await
        .expect("Contextual reviews did not reach DHT consistency");

    let commerce_reviews: Vec<Record> = conductors[2]
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

    assert_eq!(commerce_reviews.len(), 2);

    let professional_reviews: Vec<Record> = conductors[2]
        .sweet_handle()
        .call(
            &reputation_subject,
            "get_contextual_reviews",
            GetContextualReviewsInput {
                subject_agent: subject.clone(),
                domain: Some(ReputationDomain::Professional),
            },
        )
        .await;

    assert_eq!(professional_reviews.len(), 1);

    let summaries: Vec<ContextualReputationSummary> = conductors[2]
        .sweet_handle()
        .call(
            &reputation_subject,
            "get_contextual_reputation_summary",
            GetContextualReviewsInput {
                subject_agent: subject.clone(),
                domain: None,
            },
        )
        .await;

    let commerce_summary = summaries
        .iter()
        .find(|summary| summary.domain == ReputationDomain::Commerce)
        .expect("Commerce summary missing");

    assert_eq!(commerce_summary.review_count, 2);
    assert_eq!(commerce_summary.average_rating, 4.0);

    let professional_summary = summaries
        .iter()
        .find(|summary| summary.domain == ReputationDomain::Professional)
        .expect("Professional summary missing");

    assert_eq!(professional_summary.review_count, 1);
    assert_eq!(professional_summary.average_rating, 4.0);

    let empty_domain = summaries
        .iter()
        .find(|summary| summary.domain == ReputationDomain::Hospitality)
        .expect("Empty domain summary missing");

    assert_eq!(empty_domain.review_count, 0);
    assert_eq!(empty_domain.average_rating, 0.0);

    let revoked_evidence: ActionHash = conductors[1]
        .sweet_handle()
        .call(
            &attestation_b,
            "create_attestation",
            CreateAttestationInput {
                subject: subject.clone(),
                claim_type: ClaimType::Skill,
                claim: "Evidence to be revoked".into(),
                evidence: Vec::<EvidenceReference>::new(),
                expires_at: None,
                supersedes: None,
            },
        )
        .await;

    await_consistency(apps.iter().flat_map(|app| app.cells().iter()))
        .await
        .expect("Revocable evidence did not reach DHT consistency");

    let _: ActionHash = conductors[1]
        .sweet_handle()
        .call(
            &attestation_b,
            "revoke_attestation",
            RevokeAttestationInput {
                attestation: revoked_evidence.clone(),
                reason: Some("Integration test revocation".into()),
            },
        )
        .await;

    await_consistency(apps.iter().flat_map(|app| app.cells().iter()))
        .await
        .expect("Evidence revocation did not reach DHT consistency");

    let inactive_evidence_review: Result<ActionHash, _> = conductors[0]
        .sweet_handle()
        .call_fallible(
            &reputation_a,
            "create_contextual_review",
            CreateContextualReviewInput {
                subject_agent: subject,
                domain: ReputationDomain::Reliability,
                rating: 5,
                comment: Some("Must be rejected".into()),
                evidence_attestations: vec![revoked_evidence],
            },
        )
        .await;

    assert!(
        inactive_evidence_review.is_err(),
        "A review referencing revoked evidence must be rejected"
    );
}
