use hdk::prelude::*;
use reputation_integrity::{
    validate_contextual_review, ContextualReview, EntryTypes, LinkTypes,
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
enum AttestationStatus {
    Active,
    Revoked,
    Expired,
    Superseded,
}

fn ensure_active_evidence(evidence: &[ActionHash]) -> ExternResult<()> {
    for action_hash in evidence {
        let response = call(
            CallTargetCell::Local,
            "attestation_coordinator",
            "resolve_attestation_status".into(),
            None,
            ResolveAttestationStatusInput {
                action_hash: action_hash.clone(),
            },
        )?;

        let status: AttestationStatus = match response {
            ZomeCallResponse::Ok(output) => output
                .decode()
                .map_err(|error| wasm_error!(error.to_string()))?,
            _ => {
                return Err(wasm_error!(WasmErrorInner::Guest(
                    "Attestation status could not be verified".into()
                )));
            }
        };

        if status != AttestationStatus::Active {
            return Err(wasm_error!(WasmErrorInner::Guest(
                format!(
                    "Evidence {:?} is not active: {:?}",
                    action_hash, status
                )
            )));
        }
    }

    Ok(())
}

#[hdk_extern]
pub fn create_contextual_review(
    input: CreateContextualReviewInput,
) -> ExternResult<ActionHash> {
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
pub fn get_contextual_reviews(
    input: GetContextualReviewsInput,
) -> ExternResult<Vec<Record>> {
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

        ensure_active_evidence(&review.evidence_attestations)?;

        records.push(record);
    }

    records.sort_by_key(|record| record.action().timestamp());

    Ok(records)
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
