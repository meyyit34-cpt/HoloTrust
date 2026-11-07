use hdk::prelude::*;
use reputation_integrity::{
    validate_contextual_review, Attestation, ContextualReview, EntryTypes, LinkTypes,
    ReputationDomain,
};

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateContextualReviewInput {
    pub subject_agent: AgentPubKey,
    pub domain: ReputationDomain,
    pub rating: u8,
    pub comment: Option<String>,
    pub evidence_attestations: Vec<ActionHash>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetContextualReviewsInput {
    pub subject_agent: AgentPubKey,
    pub domain: Option<ReputationDomain>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ContextualReputationSummary {
    pub domain: ReputationDomain,
    pub review_count: u32,
    pub average_rating: f64,
}

#[derive(Serialize, Deserialize, Debug)]
struct ResolveAttestationStatusInput {
    action_hash: ActionHash,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum EvidenceStatus {
    Active,
    Revoked,
    Expired,
    Superseded,
    Missing,
    Invalid,
    Unavailable,
}

#[derive(Serialize, Deserialize)]
pub struct EvidenceProvenance {
    pub action_hash: ActionHash,
    pub status: EvidenceStatus,
    pub issuer: Option<AgentPubKey>,
    pub issued_at: Option<Timestamp>,
    pub attestation: Option<Attestation>,
}

#[derive(Serialize, Deserialize)]
pub struct ContextualReviewProvenance {
    pub action_hash: ActionHash,
    pub author: AgentPubKey,
    pub timestamp: Timestamp,
    pub review: ContextualReview,
    pub evidence: Vec<EvidenceProvenance>,
}

#[derive(Serialize, Deserialize, Debug)]
struct GetAttestationInput {
    action_hash: ActionHash,
}

fn get_attestation_status(action_hash: &ActionHash) -> ExternResult<EvidenceStatus> {
    let response = call(
        CallTargetCell::Local,
        "attestation_coordinator",
        "resolve_attestation_status".into(),
        None,
        ResolveAttestationStatusInput {
            action_hash: action_hash.clone(),
        },
    )?;

    match response {
        ZomeCallResponse::Ok(output) => output
            .decode()
            .map_err(|error| wasm_error!(error.to_string())),
        _ => Err(wasm_error!(WasmErrorInner::Guest(
            "Attestation status could not be verified".into()
        ))),
    }
}

fn get_attestation_record(action_hash: &ActionHash) -> ExternResult<Option<Record>> {
    let response = call(
        CallTargetCell::Local,
        "attestation_coordinator",
        "get_attestation".into(),
        None,
        GetAttestationInput {
            action_hash: action_hash.clone(),
        },
    )?;

    match response {
        ZomeCallResponse::Ok(output) => output
            .decode()
            .map_err(|error| wasm_error!(error.to_string())),
        _ => Err(wasm_error!(WasmErrorInner::Guest(
            "Attestation provenance could not be retrieved".into()
        ))),
    }
}

fn get_evidence_provenance(action_hash: &ActionHash) -> ExternResult<EvidenceProvenance> {
    let record = match get_attestation_record(action_hash) {
        Ok(Some(record)) => record,
        Ok(None) => {
            let raw_record = get(action_hash.clone(), GetOptions::default())?;
            return Ok(EvidenceProvenance {
                action_hash: action_hash.clone(),
                status: if raw_record.is_some() {
                    EvidenceStatus::Invalid
                } else {
                    EvidenceStatus::Missing
                },
                issuer: None,
                issued_at: None,
                attestation: None,
            });
        }
        Err(_) => {
            return Ok(EvidenceProvenance {
                action_hash: action_hash.clone(),
                status: EvidenceStatus::Unavailable,
                issuer: None,
                issued_at: None,
                attestation: None,
            });
        }
    };

    let attestation = match record.entry().to_app_option::<Attestation>() {
        Ok(Some(attestation)) => attestation,
        _ => {
            return Ok(EvidenceProvenance {
                action_hash: action_hash.clone(),
                status: EvidenceStatus::Invalid,
                issuer: Some(record.action().author().clone()),
                issued_at: Some(record.action().timestamp()),
                attestation: None,
            });
        }
    };

    let status = get_attestation_status(action_hash)
        .unwrap_or(EvidenceStatus::Unavailable);

    Ok(EvidenceProvenance {
        action_hash: action_hash.clone(),
        status,
        issuer: Some(record.action().author().clone()),
        issued_at: Some(record.action().timestamp()),
        attestation: Some(attestation),
    })
}

fn evidence_is_active(evidence: &[ActionHash]) -> ExternResult<bool> {
    for action_hash in evidence {
        let status = get_attestation_status(action_hash)?;

        if status != EvidenceStatus::Active {
            return Ok(false);
        }
    }

    Ok(true)
}

fn ensure_active_evidence(evidence: &[ActionHash]) -> ExternResult<()> {
    for action_hash in evidence {
        let status = get_attestation_status(action_hash)?;

        if status != EvidenceStatus::Active {
            return Err(wasm_error!(WasmErrorInner::Guest(format!(
                "Evidence {:?} is not active: {:?}",
                action_hash, status
            ))));
        }
    }

    Ok(())
}

#[hdk_extern]
pub fn create_contextual_review(input: CreateContextualReviewInput) -> ExternResult<ActionHash> {
    let author = agent_info()?.agent_initial_pubkey;

    if author == input.subject_agent {
        return Err(wasm_error!(WasmErrorInner::Guest(
            "You cannot review your own agent".into()
        )));
    }

    let review = ContextualReview {
        schema_version: 1,
        subject_agent: input.subject_agent.clone(),
        domain: input.domain,
        rating: input.rating,
        comment: input.comment,
        evidence_attestations: input.evidence_attestations,
    };

    validate_contextual_review(&review)
        .map_err(|reason| wasm_error!(WasmErrorInner::Guest(reason)))?;

    ensure_active_evidence(&review.evidence_attestations)?;

    let review_hash = create_entry(EntryTypes::ContextualReview(review))?;

    create_link(
        input.subject_agent,
        review_hash.clone(),
        LinkTypes::SubjectToContextualReview,
        "review",
    )?;

    Ok(review_hash)
}

#[hdk_extern]
pub fn get_contextual_reviews(input: GetContextualReviewsInput) -> ExternResult<Vec<Record>> {
    let links = get_links(
        LinkQuery::try_new(
            input.subject_agent.clone(),
            LinkTypes::SubjectToContextualReview,
        )?,
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

        if record.action().author() == &input.subject_agent {
            continue;
        }

        let review = match record.entry().to_app_option::<ContextualReview>() {
            Ok(Some(review)) => review,
            _ => continue,
        };

        if review.subject_agent != input.subject_agent {
            continue;
        }

        if let Some(domain) = &input.domain {
            if &review.domain != domain {
                continue;
            }
        }

        if validate_contextual_review(&review).is_err() {
            continue;
        }

        // Historical reviews remain visible even if their evidence later
        // becomes revoked, expired, or superseded.
        records.push(record);
    }

    records.sort_by_key(|record| record.action().timestamp());

    Ok(records)
}

#[hdk_extern]
pub fn get_contextual_reviews_with_provenance(
    input: GetContextualReviewsInput,
) -> ExternResult<Vec<ContextualReviewProvenance>> {
    let records = get_contextual_reviews(input)?;
    let mut results = Vec::with_capacity(records.len());

    for record in records {
        let review = record
            .entry()
            .to_app_option::<ContextualReview>()
            .map_err(|error| wasm_error!(WasmErrorInner::Guest(error.to_string())))?
            .ok_or_else(|| {
                wasm_error!(WasmErrorInner::Guest(
                    "Contextual review record could not be decoded".into()
                ))
            })?;

        let mut evidence = Vec::with_capacity(review.evidence_attestations.len());

        for action_hash in &review.evidence_attestations {
            evidence.push(get_evidence_provenance(action_hash)?);
        }

        results.push(ContextualReviewProvenance {
            action_hash: record.action_address().clone(),
            author: record.action().author().clone(),
            timestamp: record.action().timestamp(),
            review,
            evidence,
        });
    }

    Ok(results)
}

#[hdk_extern]
pub fn get_contextual_reputation_summary(
    input: GetContextualReviewsInput,
) -> ExternResult<Vec<ContextualReputationSummary>> {
    let records = get_contextual_reviews(GetContextualReviewsInput {
        subject_agent: input.subject_agent,
        domain: None,
    })?;

    let domains = vec![
        ReputationDomain::Commerce,
        ReputationDomain::Professional,
        ReputationDomain::Hospitality,
        ReputationDomain::Technical,
        ReputationDomain::Community,
        ReputationDomain::Reliability,
    ];

    let mut summaries = Vec::new();

    for domain in domains {
        let mut total_rating: u64 = 0;
        let mut review_count: u32 = 0;

        for record in &records {
            let review = match record.entry().to_app_option::<ContextualReview>() {
                Ok(Some(review)) => review,
                _ => continue,
            };

            if review.domain != domain {
                continue;
            }

            if validate_contextual_review(&review).is_err() {
                continue;
            }

            // Only currently active evidence contributes to the score.
            // The original review itself remains in the historical record.
            if !evidence_is_active(&review.evidence_attestations)? {
                continue;
            }

            total_rating += u64::from(review.rating);
            review_count += 1;
        }

        let average_rating = if review_count == 0 {
            0.0
        } else {
            total_rating as f64 / f64::from(review_count)
        };

        summaries.push(ContextualReputationSummary {
            domain,
            review_count,
            average_rating,
        });
    }

    Ok(summaries)
}
