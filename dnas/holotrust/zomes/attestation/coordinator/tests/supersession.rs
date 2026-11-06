use attestation_coordinator::{
    AttestationStatus, AttestationVerification, CreateAttestationInput, GetAttestationInput,
    GetAttestationsForSubjectInput, RevokeAttestationInput, VerifyAttestationInput,
};
use attestation_integrity::{ClaimType, EvidenceReference};
use holochain::prelude::ActionHash;
use holochain::sweettest::*;
use std::path::PathBuf;

fn attestation_input(
    subject: holochain::prelude::AgentPubKey,
    claim_type: ClaimType,
    claim: &str,
    supersedes: Option<ActionHash>,
) -> CreateAttestationInput {
    CreateAttestationInput {
        subject,
        claim_type,
        claim: claim.to_string(),
        evidence: Vec::<EvidenceReference>::new(),
        expires_at: None,
        supersedes,
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn supersession_and_revocation_require_authorization_and_preserve_history() {
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

    let issuer_a = apps[0].agent().clone();
    let issuer_b = apps[1].agent().clone();
    let subject = apps[2].agent().clone();

    let issuer_a_zome = apps[0].cells()[0].zome("attestation_coordinator");
    let issuer_b_zome = apps[1].cells()[0].zome("attestation_coordinator");
    let subject_zome = apps[2].cells()[0].zome("attestation_coordinator");

    let original: ActionHash = conductors[0]
        .sweet_handle()
        .call(
            &issuer_a_zome,
            "create_attestation",
            attestation_input(subject.clone(), ClaimType::Skill, "Rust development", None),
        )
        .await;

    await_consistency(apps.iter().flat_map(|app| app.cells().iter()))
        .await
        .expect("Original attestation did not reach DHT consistency");

    // A third agent must be able to discover and retrieve the original attestation.
    let subject_attestations = conductors[2]
        .sweet_handle()
        .call(
            &subject_zome,
            "get_attestations_for_subject",
            GetAttestationsForSubjectInput {
                subject: subject.clone(),
            },
        )
        .await;

    assert!(
        !subject_attestations.is_empty(),
        "A third agent must discover the subject's original attestation"
    );

    let discovered_original = conductors[2]
        .sweet_handle()
        .call(
            &subject_zome,
            "get_attestation",
            GetAttestationInput {
                action_hash: original.clone(),
            },
        )
        .await;

    assert!(
        discovered_original.is_some(),
        "A third agent must retrieve the original attestation"
    );

    // Verify the original attestation from an independent agent.
    let third_party_verification: AttestationVerification = conductors[2]
        .sweet_handle()
        .call(
            &subject_zome,
            "verify_attestation",
            VerifyAttestationInput {
                action_hash: original.clone(),
            },
        )
        .await;

    assert_eq!(third_party_verification.attestation, original);
    assert_eq!(third_party_verification.issuer, issuer_a);
    assert_eq!(third_party_verification.subject, subject);
    assert_eq!(third_party_verification.claim_type, ClaimType::Skill);
    assert_eq!(third_party_verification.claim, "Rust development");
    assert_eq!(third_party_verification.evidence_count, 0);

    let replacement: ActionHash = conductors[0]
        .sweet_handle()
        .call(
            &issuer_a_zome,
            "create_attestation",
            attestation_input(
                subject.clone(),
                ClaimType::Skill,
                "Advanced Rust development",
                Some(original.clone()),
            ),
        )
        .await;

    assert_ne!(replacement, original);

    await_consistency(apps.iter().flat_map(|app| app.cells().iter()))
        .await
        .expect("Replacement attestation did not reach DHT consistency");

    // The independent agent must discover both historical attestations.
    let updated_subject_attestations = conductors[2]
        .sweet_handle()
        .call(
            &subject_zome,
            "get_attestations_for_subject",
            GetAttestationsForSubjectInput {
                subject: subject.clone(),
            },
        )
        .await;

    assert!(
        updated_subject_attestations.len() >= 2,
        "A third agent must discover both original and replacement attestations"
    );

    let third_party_replacement: AttestationVerification = conductors[2]
        .sweet_handle()
        .call(
            &subject_zome,
            "verify_attestation",
            VerifyAttestationInput {
                action_hash: replacement.clone(),
            },
        )
        .await;

    assert_eq!(third_party_replacement.attestation, replacement);
    assert_eq!(third_party_replacement.issuer, issuer_a);
    assert_eq!(third_party_replacement.subject, subject);
    assert_eq!(
        third_party_replacement.claim,
        "Advanced Rust development"
    );

    let cross_issuer: Result<ActionHash, _> = conductors[1]
        .sweet_handle()
        .call_fallible(
            &issuer_b_zome,
            "create_attestation",
            attestation_input(
                subject.clone(),
                ClaimType::Skill,
                "Another issuer replacement",
                Some(original.clone()),
            ),
        )
        .await;

    assert!(
        cross_issuer.is_err(),
        "A different issuer must not supersede another issuer's attestation"
    );

    let different_subject: Result<ActionHash, _> = conductors[0]
        .sweet_handle()
        .call_fallible(
            &issuer_a_zome,
            "create_attestation",
            attestation_input(
                issuer_b.clone(),
                ClaimType::Skill,
                "Different subject replacement",
                Some(original.clone()),
            ),
        )
        .await;

    assert!(
        different_subject.is_err(),
        "Supersession must preserve the original subject"
    );

    let different_claim_type: Result<ActionHash, _> = conductors[0]
        .sweet_handle()
        .call_fallible(
            &issuer_a_zome,
            "create_attestation",
            attestation_input(
                subject.clone(),
                ClaimType::Experience,
                "Different claim type replacement",
                Some(original.clone()),
            ),
        )
        .await;

    assert!(
        different_claim_type.is_err(),
        "Supersession must preserve the original claim type"
    );

    let original_status: AttestationStatus = conductors[0]
        .sweet_handle()
        .call(
            &issuer_a_zome,
            "resolve_attestation_status",
            GetAttestationInput {
                action_hash: original.clone(),
            },
        )
        .await;

    assert_eq!(
        original_status,
        AttestationStatus::Superseded,
        "The original attestation should resolve as superseded"
    );

    // The independent agent must also resolve the original as superseded.
    let third_party_original_status: AttestationStatus = conductors[2]
        .sweet_handle()
        .call(
            &subject_zome,
            "resolve_attestation_status",
            GetAttestationInput {
                action_hash: original.clone(),
            },
        )
        .await;

    assert_eq!(
        third_party_original_status,
        AttestationStatus::Superseded,
        "A third agent must recognize the original attestation as superseded"
    );

    let replacement_status: AttestationStatus = conductors[0]
        .sweet_handle()
        .call(
            &issuer_a_zome,
            "resolve_attestation_status",
            GetAttestationInput {
                action_hash: replacement.clone(),
            },
        )
        .await;

    assert_eq!(
        replacement_status,
        AttestationStatus::Active,
        "The replacement attestation should initially remain active"
    );

    let unauthorized_revocation: Result<ActionHash, _> = conductors[1]
        .sweet_handle()
        .call_fallible(
            &issuer_b_zome,
            "revoke_attestation",
            RevokeAttestationInput {
                attestation: replacement.clone(),
                reason: Some("Unauthorized revocation attempt".into()),
            },
        )
        .await;

    assert!(
        unauthorized_revocation.is_err(),
        "A different issuer must not revoke another issuer's attestation"
    );

    let revocation: ActionHash = conductors[0]
        .sweet_handle()
        .call(
            &issuer_a_zome,
            "revoke_attestation",
            RevokeAttestationInput {
                attestation: replacement.clone(),
                reason: Some("Issuer-requested revocation".into()),
            },
        )
        .await;

    assert_ne!(revocation, replacement);

    await_consistency(apps.iter().flat_map(|app| app.cells().iter()))
        .await
        .expect("Revocation did not reach DHT consistency");

    let duplicate_revocation: Result<ActionHash, _> = conductors[0]
        .sweet_handle()
        .call_fallible(
            &issuer_a_zome,
            "revoke_attestation",
            RevokeAttestationInput {
                attestation: replacement.clone(),
                reason: Some("Duplicate revocation attempt".into()),
            },
        )
        .await;

    assert!(
        duplicate_revocation.is_err(),
        "Duplicate revocation must be rejected"
    );

    let revoked_status: AttestationStatus = conductors[0]
        .sweet_handle()
        .call(
            &issuer_a_zome,
            "resolve_attestation_status",
            GetAttestationInput {
                action_hash: replacement.clone(),
            },
        )
        .await;

    assert_eq!(
        revoked_status,
        AttestationStatus::Revoked,
        "A revoked attestation must resolve as revoked"
    );

    // The independent agent must observe the revocation through the DHT.
    let third_party_revoked_status: AttestationStatus = conductors[2]
        .sweet_handle()
        .call(
            &subject_zome,
            "resolve_attestation_status",
            GetAttestationInput {
                action_hash: replacement,
            },
        )
        .await;

    assert_eq!(
        third_party_revoked_status,
        AttestationStatus::Revoked,
        "A third agent must recognize the replacement as revoked"
    );
}
