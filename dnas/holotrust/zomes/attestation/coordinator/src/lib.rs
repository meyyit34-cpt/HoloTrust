use attestation_integrity::{
    validate_attestation, validate_revocation, Attestation, AttestationRevocation, ClaimType,
    EntryTypes, EvidenceReference, LinkTypes,
};
use hdk::prelude::*;

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateAttestationInput {
    pub subject: AgentPubKey,
    pub claim_type: ClaimType,
    pub claim: String,
    pub evidence: Vec<EvidenceReference>,
    pub expires_at: Option<Timestamp>,
    pub supersedes: Option<ActionHash>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetAttestationInput {
    pub action_hash: ActionHash,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetAttestationsForSubjectInput {
    pub subject: AgentPubKey,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetAttestationsByIssuerInput {
    pub issuer: AgentPubKey,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RevokeAttestationInput {
    pub attestation: ActionHash,
    pub reason: Option<String>,
}

#[hdk_extern]
pub fn create_attestation(input: CreateAttestationInput) -> ExternResult<ActionHash> {
    let attestation = Attestation {
        schema_version: 1,
        subject: input.subject.clone(),
        claim_type: input.claim_type,
        claim: input.claim,
        evidence: input.evidence,
        expires_at: input.expires_at,
        supersedes: input.supersedes,
    };

    validate_attestation(&attestation)
        .map_err(|reason| wasm_error!(WasmErrorInner::Guest(reason)))?;

    let action_hash = create_entry(EntryTypes::Attestation(attestation))?;
    let issuer = agent_info()?.agent_initial_pubkey;

    create_link(
        input.subject,
        action_hash.clone(),
        LinkTypes::SubjectToAttestation,
        "attestation",
    )?;

    create_link(
        issuer,
        action_hash.clone(),
        LinkTypes::IssuerToAttestation,
        "attestation",
    )?;

    Ok(action_hash)
}

#[hdk_extern]
pub fn get_attestation(input: GetAttestationInput) -> ExternResult<Option<Record>> {
    let Some(record) = get(input.action_hash, GetOptions::default())? else {
        return Ok(None);
    };

    let attestation = match record.entry().to_app_option::<Attestation>() {
        Ok(Some(attestation)) => attestation,
        _ => return Ok(None),
    };

    if validate_attestation(&attestation).is_err() {
        return Ok(None);
    }

    Ok(Some(record))
}

#[hdk_extern]
pub fn get_attestations_for_subject(
    input: GetAttestationsForSubjectInput,
) -> ExternResult<Vec<Record>> {
    let links = get_links(
        LinkQuery::try_new(input.subject.clone(), LinkTypes::SubjectToAttestation)?,
        GetStrategy::default(),
    )?;

    let mut records = Vec::new();

    for link in links {
        let Some(action_hash) = link.target.into_action_hash() else {
            continue;
        };

        let Some(record) = get(action_hash, GetOptions::default())? else {
            continue;
        };

        if record.action().author() != &link.author {
            continue;
        }

        let attestation = match record.entry().to_app_option::<Attestation>() {
            Ok(Some(attestation)) => attestation,
            _ => continue,
        };

        if attestation.subject != input.subject || validate_attestation(&attestation).is_err() {
            continue;
        }

        records.push(record);
    }

    Ok(records)
}

#[hdk_extern]
pub fn get_attestations_by_issuer(
    input: GetAttestationsByIssuerInput,
) -> ExternResult<Vec<Record>> {
    let links = get_links(
        LinkQuery::try_new(input.issuer.clone(), LinkTypes::IssuerToAttestation)?,
        GetStrategy::default(),
    )?;

    let mut records = Vec::new();

    for link in links {
        if link.author != input.issuer {
            continue;
        }

        let Some(action_hash) = link.target.into_action_hash() else {
            continue;
        };

        let Some(record) = get(action_hash, GetOptions::default())? else {
            continue;
        };

        if record.action().author() != &input.issuer {
            continue;
        }

        let attestation = match record.entry().to_app_option::<Attestation>() {
            Ok(Some(attestation)) => attestation,
            _ => continue,
        };

        if validate_attestation(&attestation).is_err() {
            continue;
        }

        records.push(record);
    }

    Ok(records)
}

#[hdk_extern]
pub fn revoke_attestation(input: RevokeAttestationInput) -> ExternResult<ActionHash> {
    let Some(record) = get(input.attestation.clone(), GetOptions::default())? else {
        return Err(wasm_error!(WasmErrorInner::Guest(
            "Attestation not found".into(),
        )));
    };

    let target = record
        .entry()
        .to_app_option::<Attestation>()
        .map_err(|error| wasm_error!(WasmErrorInner::Guest(error.to_string())))?;

    if target.is_none() {
        return Err(wasm_error!(WasmErrorInner::Guest(
            "Target record is not an attestation".into(),
        )));
    }

    let revocation = AttestationRevocation {
        schema_version: 1,
        attestation: input.attestation,
        reason: input.reason,
    };

    validate_revocation(&revocation)
        .map_err(|reason| wasm_error!(WasmErrorInner::Guest(reason)))?;

    create_entry(EntryTypes::AttestationRevocation(revocation))
}
