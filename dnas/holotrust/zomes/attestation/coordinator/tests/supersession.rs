use attestation_coordinator::{
    AttestationStatus, CreateAttestationInput, GetAttestationInput, RevokeAttestationInput,
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

    let mut conductors = SweetConductorBatch::standard(3).await;

    let apps = conductors
        .setup_app("holotrust", [&dna])
        .await
        .expect("Failed to install apps");

    conductors.exchange_peer_info().await;

    let issuer_b = apps[1].agent().clone();
    let subject = apps[2].agent().clone();

    let issuer_a_zome = apps[0].cells()[0].zome("attestation_coordinator");
    let issuer_b_zome = apps[1].cells()[0].zome("attestation_coordinator");

    // Issuer A creates the original attestation.
    let original: ActionHash = conductors[0]
        .sweet_handle()
        .call(
            &issuer_a_zome,
            "create_attestation",
            attestation_input(
                subject.clone(),
                ClaimType::Skill,
                "Rust development",
                None,
            ),
        )
        .await;

    await_consistency(apps.iter().flat_map(|app| app.cells().iter()))
        .await
        .expect("Original attestation did not reach DHT consistency");

    // Issuer A supersedes its own attestation.
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

    // Another issuer cannot supersede Issuer A's attestation.
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

    // Supersession cannot change the subject.
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

    // Supersession cannot change the claim type.
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

    // The original record must resolve as superseded.
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

    // The replacement must initially remain active.
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

    // A different issuer cannot revoke Issuer A's replacement.
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

    // The original issuer can revoke its own replacement.
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

    // The same attestation cannot be revoked twice.
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

    // Revocation takes precedence in status resolution.
    let revoked_status: AttestationStatus = conductors[0]
        .sweet_handle()
        .call(
            &issuer_a_zome,
            "resolve_attestation_status",
            GetAttestationInput {
                action_hash: replacement,
            },
        )
        .await;

    assert_eq!(
        revoked_status,
        AttestationStatus::Revoked,
        "A revoked attestation must resolve as revoked"
    );
}
