
use attestation_coordinator::{
    AttestationStatus, CreateAttestationInput, GetAttestationInput,
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
async fn supersession_requires_same_issuer_subject_and_claim_type() {
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

    let issuer_b = apps[1].agent().clone();
    let subject = apps[2].agent().clone();

    let issuer_a_zome = apps[0].cells()[0].zome("attestation_coordinator");
    let issuer_b_zome = apps[1].cells()[0].zome("attestation_coordinator");

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
                issuer_b,
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
                subject,
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
                action_hash: original,
            },
        )
        .await;

    assert_eq!(
        original_status,
        AttestationStatus::Superseded,
        "The original attestation should resolve as superseded"
    );

    let replacement_status: AttestationStatus = conductors[0]
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
        replacement_status,
        AttestationStatus::Active,
        "The replacement attestation should remain active"
    );
}

// END OF FILE
