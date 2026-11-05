use attestation_integrity::{
    validate_attestation, validate_revocation, Attestation, AttestationRevocation, ClaimType,
    EvidenceReference,
};
use hdi::prelude::*;

fn make_attestation(claim: &str) -> Attestation {
    Attestation {
        schema_version: 1,
        subject: AgentPubKey::from_raw_36(vec![0; 36]),
        claim_type: ClaimType::Skill,
        claim: claim.to_string(),
        evidence: Vec::new(),
        expires_at: None,
        supersedes: None,
    }
}

fn make_revocation(reason: Option<&str>) -> AttestationRevocation {
    AttestationRevocation {
        schema_version: 1,
        attestation: ActionHash::from_raw_36(vec![0; 36]),
        reason: reason.map(str::to_string),
    }
}

#[test]
fn accepts_valid_attestation() {
    assert!(validate_attestation(&make_attestation("Rust developer")).is_ok());
}

#[test]
fn rejects_unsupported_attestation_schema() {
    let mut attestation = make_attestation("Rust developer");
    attestation.schema_version = 2;
    assert!(validate_attestation(&attestation).is_err());
}

#[test]
fn rejects_empty_claim() {
    assert!(validate_attestation(&make_attestation("")).is_err());
}

#[test]
fn rejects_whitespace_only_claim() {
    assert!(validate_attestation(&make_attestation("   ")).is_err());
}

#[test]
fn accepts_claim_with_1000_characters() {
    let claim = "a".repeat(1000);
    assert!(validate_attestation(&make_attestation(&claim)).is_ok());
}

#[test]
fn rejects_claim_with_1001_characters() {
    let claim = "a".repeat(1001);
    assert!(validate_attestation(&make_attestation(&claim)).is_err());
}

#[test]
fn rejects_control_characters_in_claim() {
    assert!(validate_attestation(&make_attestation("Valid\nInvalid")).is_err());
}

#[test]
fn accepts_unicode_claim() {
    assert!(validate_attestation(&make_attestation("Türkçe yetkinlik ve başarı")).is_ok());
}

#[test]
fn accepts_five_evidence_references() {
    let mut attestation = make_attestation("Verified skill");
    attestation.evidence = vec![
        EvidenceReference::External {
            url: "https://example.com/evidence".into(),
            sha256: "a".repeat(64),
        };
        5
    ];

    assert!(validate_attestation(&attestation).is_ok());
}

#[test]
fn rejects_more_than_five_evidence_references() {
    let mut attestation = make_attestation("Verified skill");
    attestation.evidence = vec![
        EvidenceReference::External {
            url: "https://example.com/evidence".into(),
            sha256: "a".repeat(64),
        };
        6
    ];

    assert!(validate_attestation(&attestation).is_err());
}

#[test]
fn rejects_non_https_evidence_url() {
    let mut attestation = make_attestation("Verified skill");
    attestation.evidence.push(EvidenceReference::External {
        url: "http://example.com/evidence".into(),
        sha256: "a".repeat(64),
    });

    assert!(validate_attestation(&attestation).is_err());
}

#[test]
fn rejects_invalid_evidence_digest() {
    let mut attestation = make_attestation("Verified skill");
    attestation.evidence.push(EvidenceReference::External {
        url: "https://example.com/evidence".into(),
        sha256: "invalid".into(),
    });

    assert!(validate_attestation(&attestation).is_err());
}

#[test]
fn rejects_unsupported_revocation_schema() {
    let mut revocation = make_revocation(None);
    revocation.schema_version = 2;

    assert!(validate_revocation(&revocation).is_err());
}

#[test]
fn accepts_valid_revocation() {
    assert!(validate_revocation(&make_revocation(Some("Incorrect claim"))).is_ok());
}

#[test]
fn accepts_revocation_without_reason() {
    assert!(validate_revocation(&make_revocation(None)).is_ok());
}

#[test]
fn accepts_revocation_reason_with_500_characters() {
    let reason = "a".repeat(500);
    assert!(validate_revocation(&make_revocation(Some(&reason))).is_ok());
}

#[test]
fn rejects_revocation_reason_with_501_characters() {
    let reason = "a".repeat(501);
    assert!(validate_revocation(&make_revocation(Some(&reason))).is_err());
}

#[test]
fn rejects_control_characters_in_revocation_reason() {
    assert!(validate_revocation(&make_revocation(Some("Invalid\nreason"))).is_err());
}
