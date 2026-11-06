#[derive(Serialize, Deserialize, Debug)]
pub struct VerifyAttestationInput {
    pub action_hash: ActionHash,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct AttestationVerification {
    pub attestation: ActionHash,
    pub issuer: AgentPubKey,
    pub subject: AgentPubKey,
    pub claim_type: ClaimType,
    pub claim: String,
    pub issued_at: Timestamp,
    pub evidence_count: usize,
}

#[hdk_extern]
pub fn verify_attestation(
    input: VerifyAttestationInput,
) -> ExternResult<AttestationVerification> {
    let Some(record) = get(input.action_hash.clone(), GetOptions::default())? else {
        return Err(wasm_error!(WasmErrorInner::Guest(
            "Attestation not found".into(),
        )));
    };

    let attestation = record
        .entry()
        .to_app_option::<Attestation>()
        .map_err(|error| wasm_error!(WasmErrorInner::Guest(error.to_string())))?
        .ok_or_else(|| {
            wasm_error!(WasmErrorInner::Guest(
                "Target record is not an attestation".into(),
            ))
        })?;

    validate_attestation(&attestation)
        .map_err(|reason| wasm_error!(WasmErrorInner::Guest(reason)))?;

    let issued_at = record.action().timestamp();

    if let Some(expires_at) = &attestation.expires_at {
        if expires_at <= &issued_at {
            return Err(wasm_error!(WasmErrorInner::Guest(
                "Attestation expiry must be later than its issuance time".into(),
            )));
        }
    }

    Ok(AttestationVerification {
        attestation: input.action_hash,
        issuer: record.action().author().clone(),
        subject: attestation.subject,
        claim_type: attestation.claim_type,
        claim: attestation.claim,
        issued_at,
        evidence_count: attestation.evidence.len(),
    })
}
