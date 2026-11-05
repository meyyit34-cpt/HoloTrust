use hdi::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ClaimType {
    Skill,
    Experience,
    ServiceCompletion,
    Membership,
    Credential,
    Other,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum EvidenceReference {
    Action(ActionHash),
    Entry(EntryHash),
    External { url: String, sha256: String },
}

#[hdk_entry_helper]
pub struct Attestation {
    pub schema_version: u8,
    pub subject: AgentPubKey,
    pub claim_type: ClaimType,
    pub claim: String,
    pub evidence: Vec<EvidenceReference>,
    pub expires_at: Option<Timestamp>,
    pub supersedes: Option<ActionHash>,
}

#[hdk_entry_helper]
pub struct AttestationRevocation {
    pub schema_version: u8,
    pub attestation: ActionHash,
    pub reason: Option<String>,
}

#[hdk_entry_types]
#[unit_enum(UnitEntryTypes)]
pub enum EntryTypes {
    Attestation(Attestation),
    AttestationRevocation(AttestationRevocation),
}

#[hdk_link_types]
pub enum LinkTypes {
    SubjectToAttestation,
    IssuerToAttestation,
    AttestationToRevocation,
}

pub fn validate_attestation(attestation: &Attestation) -> Result<(), String> {
    if attestation.schema_version != 1 {
        return Err("Unsupported attestation schema version".into());
    }

    let claim = attestation.claim.trim();

    if claim.is_empty() || claim.chars().count() > 1000 {
        return Err("Claim must be 1-1000 characters".into());
    }

    if claim.chars().any(char::is_control) {
        return Err("Claim contains invalid control characters".into());
    }

    if attestation.evidence.len() > 5 {
        return Err("An attestation can contain at most 5 evidence references".into());
    }

    for evidence in &attestation.evidence {
        if let EvidenceReference::External { url, sha256 } = evidence {
            if url.len() > 2048 || !url.starts_with("https://") {
                return Err("External evidence must use a valid HTTPS URL".into());
            }

            if sha256.len() != 64 || !sha256.chars().all(|c| c.is_ascii_hexdigit()) {
                return Err("Evidence digest must be a 64-character SHA-256 hash".into());
            }
        }
    }

    Ok(())
}

pub fn validate_revocation(revocation: &AttestationRevocation) -> Result<(), String> {
    if revocation.schema_version != 1 {
        return Err("Unsupported revocation schema version".into());
    }

    if let Some(reason) = &revocation.reason {
        if reason.chars().count() > 500 {
            return Err("Revocation reason cannot exceed 500 characters".into());
        }

        if reason.chars().any(char::is_control) {
            return Err("Revocation reason contains invalid control characters".into());
        }
    }

    Ok(())
}

fn validate_supersedes(
    issuer: &AgentPubKey,
    attestation: &Attestation,
) -> ExternResult<ValidateCallbackResult> {
    let Some(previous_hash) = &attestation.supersedes else {
        return Ok(ValidateCallbackResult::Valid);
    };

    let previous_record = must_get_valid_record(previous_hash.clone())?;

    if previous_record.action().author() != issuer {
        return Ok(ValidateCallbackResult::Invalid(
            "An attestation can only supersede a claim from the same issuer".into(),
        ));
    }

    let previous_attestation = previous_record
        .entry()
        .to_app_option::<Attestation>()
        .map_err(|error| wasm_error!(WasmErrorInner::Guest(error.to_string())))?;

    let Some(previous_attestation) = previous_attestation else {
        return Ok(ValidateCallbackResult::Invalid(
            "Superseded record must be an attestation".into(),
        ));
    };

    if let Err(reason) = validate_attestation(&previous_attestation) {
        return Ok(ValidateCallbackResult::Invalid(format!(
            "Superseded attestation is invalid: {reason}"
        )));
    }

    if previous_attestation.subject != attestation.subject {
        return Ok(ValidateCallbackResult::Invalid(
            "Superseded attestation must belong to the same subject".into(),
        ));
    }

    if previous_attestation.claim_type != attestation.claim_type {
        return Ok(ValidateCallbackResult::Invalid(
            "Superseded attestation must have the same claim type".into(),
        ));
    }

    Ok(ValidateCallbackResult::Valid)
}

fn validate_no_duplicate_revocation(
    author: AgentPubKey,
    previous_action_hash: ActionHash,
    revocation: &AttestationRevocation,
) -> ExternResult<ValidateCallbackResult> {
    let revocation_entry_type = EntryType::App(UnitEntryTypes::AttestationRevocation.try_into()?);

    let activity = must_get_agent_activity(author, ChainFilter::new(previous_action_hash))?;

    for item in activity {
        let previous_action = &item.action.hashed.content;

        if previous_action.entry_type() != Some(&revocation_entry_type) {
            continue;
        }

        let record = must_get_valid_record(item.action.hashed.hash.clone())?;

        let existing_revocation = record
            .entry()
            .to_app_option::<AttestationRevocation>()
            .map_err(|error| wasm_error!(WasmErrorInner::Guest(error.to_string())))?;

        if let Some(existing_revocation) = existing_revocation {
            if existing_revocation.attestation == revocation.attestation {
                return Ok(ValidateCallbackResult::Invalid(
                    "This attestation has already been revoked".into(),
                ));
            }
        }
    }

    Ok(ValidateCallbackResult::Valid)
}

#[hdk_extern]
pub fn validate(op: Op) -> ExternResult<ValidateCallbackResult> {
    let op_author = op.author().clone();
    let op_previous_action = op.prev_action().cloned();

    match op.flattened::<EntryTypes, LinkTypes>()? {
        FlatOp::CreateEntry(OpEntry::CreateEntry {
            app_entry: EntryTypes::Attestation(attestation),
            action,
        }) => {
            if let Err(reason) = validate_attestation(&attestation) {
                return Ok(ValidateCallbackResult::Invalid(reason));
            }

            validate_supersedes(action.author(), &attestation)
        }

        FlatOp::CreateEntry(OpEntry::CreateEntry {
            app_entry: EntryTypes::AttestationRevocation(revocation),
            action,
        }) => {
            if let Err(reason) = validate_revocation(&revocation) {
                return Ok(ValidateCallbackResult::Invalid(reason));
            }

            let original = must_get_valid_record(revocation.attestation.clone())?;

            let original_attestation = original
                .entry()
                .to_app_option::<Attestation>()
                .map_err(|error| wasm_error!(WasmErrorInner::Guest(error.to_string())))?;

            if original_attestation.is_none() {
                return Ok(ValidateCallbackResult::Invalid(
                    "Revocation target must be an attestation".into(),
                ));
            }

            if action.author() != original.action().author() {
                return Ok(ValidateCallbackResult::Invalid(
                    "Only the original issuer can revoke an attestation".into(),
                ));
            }

            let Some(previous_action_hash) = op_previous_action else {
                return Ok(ValidateCallbackResult::Invalid(
                    "Revocation action must have a previous source-chain action".into(),
                ));
            };

            validate_no_duplicate_revocation(op_author, previous_action_hash, &revocation)
        }

        FlatOp::Link(OpLink::CreateLink {
            link_type: LinkTypes::SubjectToAttestation,
            action,
        }) => {
            let Some(subject) = action.base_address.clone().into_agent_pub_key() else {
                return Ok(ValidateCallbackResult::Invalid(
                    "Subject link base must be an agent".into(),
                ));
            };

            let Some(target) = action.target_address.clone().into_action_hash() else {
                return Ok(ValidateCallbackResult::Invalid(
                    "Subject link target must be an attestation action hash".into(),
                ));
            };

            let record = must_get_valid_record(target)?;

            if record.action().author() != action.author() {
                return Ok(ValidateCallbackResult::Invalid(
                    "Subject link author must match the attestation issuer".into(),
                ));
            }

            let attestation = record
                .entry()
                .to_app_option::<Attestation>()
                .map_err(|error| wasm_error!(WasmErrorInner::Guest(error.to_string())))?;

            let Some(attestation) = attestation else {
                return Ok(ValidateCallbackResult::Invalid(
                    "Subject link target must contain an attestation".into(),
                ));
            };

            if attestation.subject != subject {
                return Ok(ValidateCallbackResult::Invalid(
                    "Subject link base must match the attestation subject".into(),
                ));
            }

            if let Err(reason) = validate_attestation(&attestation) {
                return Ok(ValidateCallbackResult::Invalid(reason));
            }

            Ok(ValidateCallbackResult::Valid)
        }

        FlatOp::Link(OpLink::CreateLink {
            link_type: LinkTypes::IssuerToAttestation,
            action,
        }) => {
            let Some(issuer) = action.base_address.clone().into_agent_pub_key() else {
                return Ok(ValidateCallbackResult::Invalid(
                    "Issuer link base must be an agent".into(),
                ));
            };

            if &issuer != action.author() {
                return Ok(ValidateCallbackResult::Invalid(
                    "Issuer link base must match its author".into(),
                ));
            }

            let Some(target) = action.target_address.clone().into_action_hash() else {
                return Ok(ValidateCallbackResult::Invalid(
                    "Issuer link target must be an attestation action hash".into(),
                ));
            };

            let record = must_get_valid_record(target)?;

            if record.action().author() != &issuer {
                return Ok(ValidateCallbackResult::Invalid(
                    "Issuer must match the original attestation author".into(),
                ));
            }

            let attestation = record
                .entry()
                .to_app_option::<Attestation>()
                .map_err(|error| wasm_error!(WasmErrorInner::Guest(error.to_string())))?;

            let Some(attestation) = attestation else {
                return Ok(ValidateCallbackResult::Invalid(
                    "Issuer link target must contain an attestation".into(),
                ));
            };

            if let Err(reason) = validate_attestation(&attestation) {
                return Ok(ValidateCallbackResult::Invalid(reason));
            }

            Ok(ValidateCallbackResult::Valid)
        }

        FlatOp::Link(OpLink::DeleteLink {
            link_type: LinkTypes::SubjectToAttestation | LinkTypes::IssuerToAttestation,
            ..
        }) => Ok(ValidateCallbackResult::Invalid(
            "Published attestation discovery links cannot be deleted".into(),
        )),

        FlatOp::Link(OpLink::CreateLink {
            link_type: LinkTypes::AttestationToRevocation,
            action,
        }) => {
            let Some(attestation_hash) = action.base_address.clone().into_action_hash() else {
                return Ok(ValidateCallbackResult::Invalid(
                    "Revocation link base must be an attestation action hash".into(),
                ));
            };

            let Some(revocation_hash) = action.target_address.clone().into_action_hash() else {
                return Ok(ValidateCallbackResult::Invalid(
                    "Revocation link target must be a revocation action hash".into(),
                ));
            };

            let original = must_get_valid_record(attestation_hash.clone())?;

            let attestation = original
                .entry()
                .to_app_option::<Attestation>()
                .map_err(|error| wasm_error!(WasmErrorInner::Guest(error.to_string())))?;

            if attestation.is_none() {
                return Ok(ValidateCallbackResult::Invalid(
                    "Revocation link base must reference an attestation".into(),
                ));
            }

            if original.action().author() != action.author() {
                return Ok(ValidateCallbackResult::Invalid(
                    "Only the original issuer can create the revocation link".into(),
                ));
            }

            let revocation_record = must_get_valid_record(revocation_hash)?;

            if revocation_record.action().author() != action.author() {
                return Ok(ValidateCallbackResult::Invalid(
                    "Revocation link author must match the revocation author".into(),
                ));
            }

            let revocation = revocation_record
                .entry()
                .to_app_option::<AttestationRevocation>()
                .map_err(|error| wasm_error!(WasmErrorInner::Guest(error.to_string())))?;

            let Some(revocation) = revocation else {
                return Ok(ValidateCallbackResult::Invalid(
                    "Revocation link target must contain a revocation record".into(),
                ));
            };

            if revocation.attestation != attestation_hash {
                return Ok(ValidateCallbackResult::Invalid(
                    "Revocation link must point to the attestation being revoked".into(),
                ));
            }

            Ok(ValidateCallbackResult::Valid)
        }

        FlatOp::Link(OpLink::DeleteLink {
            link_type: LinkTypes::AttestationToRevocation,
            ..
        }) => Ok(ValidateCallbackResult::Invalid(
            "Revocation history links cannot be deleted".into(),
        )),

        FlatOp::Update(OpUpdate::Entry {
            app_entry: EntryTypes::Attestation(_),
            ..
        }) => Ok(ValidateCallbackResult::Invalid(
            "Published attestations cannot be edited".into(),
        )),

        FlatOp::Update(OpUpdate::Entry {
            app_entry: EntryTypes::AttestationRevocation(_),
            ..
        }) => Ok(ValidateCallbackResult::Invalid(
            "Revocation records cannot be edited".into(),
        )),

        FlatOp::Delete(OpDelete { action }) => {
            let original = must_get_valid_record(action.deletes_address.clone())?;

            let attestation = original
                .entry()
                .to_app_option::<Attestation>()
                .map_err(|error| wasm_error!(WasmErrorInner::Guest(error.to_string())))?;

            let revocation = original
                .entry()
                .to_app_option::<AttestationRevocation>()
                .map_err(|error| wasm_error!(WasmErrorInner::Guest(error.to_string())))?;

            if attestation.is_some() || revocation.is_some() {
                return Ok(ValidateCallbackResult::Invalid(
                    "Published attestation records cannot be deleted".into(),
                ));
            }

            Ok(ValidateCallbackResult::Valid)
        }

        _ => Ok(ValidateCallbackResult::Valid),
    }
}
