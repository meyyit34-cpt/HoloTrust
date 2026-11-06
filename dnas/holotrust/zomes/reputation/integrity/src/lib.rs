
use attestation_integrity::Attestation;
use hdi::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReputationDomain {
    Commerce,
    Professional,
    Hospitality,
    Technical,
    Community,
    Reliability,
}

impl ReputationDomain {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Commerce => "commerce",
            Self::Professional => "professional",
            Self::Hospitality => "hospitality",
            Self::Technical => "technical",
            Self::Community => "community",
            Self::Reliability => "reliability",
        }
    }
}

#[hdk_entry_helper]
pub struct ContextualReview {
    pub schema_version: u8,
    pub subject_agent: AgentPubKey,
    pub domain: ReputationDomain,
    pub rating: u8,
    pub comment: Option<String>,
    pub evidence_attestations: Vec<ActionHash>,
}

#[hdk_entry_types]
#[unit_enum(UnitEntryTypes)]
pub enum EntryTypes {
    ContextualReview(ContextualReview),
}

#[hdk_link_types]
pub enum LinkTypes {
    SubjectToContextualReview,
}

pub fn validate_contextual_review(review: &ContextualReview) -> Result<(), String> {
    if review.schema_version != 1 {
        return Err("Unsupported contextual review schema version".into());
    }

    if !(1..=5).contains(&review.rating) {
        return Err("Rating must be between 1 and 5".into());
    }

    if let Some(comment) = &review.comment {
        if comment.chars().count() > 500 {
            return Err("Review comment cannot exceed 500 characters".into());
        }

        if comment.chars().any(char::is_control) {
            return Err("Review comment contains invalid control characters".into());
        }
    }

    if review.evidence_attestations.is_empty() {
        return Err("At least one evidence attestation is required".into());
    }

    if review.evidence_attestations.len() > 5 {
        return Err("A review can reference at most 5 attestations".into());
    }

    for (index, hash) in review.evidence_attestations.iter().enumerate() {
        if review.evidence_attestations[..index].contains(hash) {
            return Err("Duplicate evidence attestation references are not allowed".into());
        }
    }

    Ok(())
}

fn validate_evidence(review: &ContextualReview) -> ExternResult<ValidateCallbackResult> {
    for evidence_hash in &review.evidence_attestations {
        let record = must_get_valid_record(evidence_hash.clone())?;

        let attestation = record
            .entry()
            .to_app_option::<Attestation>()
            .map_err(|error| wasm_error!(WasmErrorInner::Guest(error.to_string())))?;

        let Some(attestation) = attestation else {
            return Ok(ValidateCallbackResult::Invalid(
                "Evidence reference must point to an attestation".into(),
            ));
        };

        if attestation.subject != review.subject_agent {
            return Ok(ValidateCallbackResult::Invalid(
                "Evidence attestation must belong to the reviewed subject".into(),
            ));
        }

        if attestation.schema_version != 1 {
            return Ok(ValidateCallbackResult::Invalid(
                "Evidence attestation has an unsupported schema version".into(),
            ));
        }
    }

    Ok(ValidateCallbackResult::Valid)
}

fn validate_review_history(
    author: &AgentPubKey,
    previous_action: Option<ActionHash>,
    review: &ContextualReview,
) -> ExternResult<ValidateCallbackResult> {
    let Some(chain_top) = previous_action else {
        return Ok(ValidateCallbackResult::Invalid(
            "A contextual review must follow an existing source-chain action".into(),
        ));
    };

    let review_entry_type = EntryType::App(UnitEntryTypes::ContextualReview.try_into()?);

    let activity = must_get_agent_activity(author.clone(), ChainFilter::new(chain_top))?;

    for item in activity {
        if item.action.hashed.content.entry_type() != Some(&review_entry_type) {
            continue;
        }

        let record = must_get_valid_record(item.action.hashed.hash.clone())?;

        let previous_review = record
            .entry()
            .to_app_option::<ContextualReview>()
            .map_err(|error| wasm_error!(WasmErrorInner::Guest(error.to_string())))?;

        let Some(previous_review) = previous_review else {
            continue;
        };

        if previous_review.subject_agent == review.subject_agent
            && previous_review.domain == review.domain
        {
            return Ok(ValidateCallbackResult::Invalid(
                "An agent can publish only one review per subject and domain".into(),
            ));
        }
    }

    Ok(ValidateCallbackResult::Valid)
}

#[hdk_extern]
pub fn validate(op: Op) -> ExternResult<ValidateCallbackResult> {
    match op.flattened::<EntryTypes, LinkTypes>()? {
        FlatOp::CreateEntry(OpEntry::CreateEntry {
            app_entry: EntryTypes::ContextualReview(review),
            action,
        }) => {
            if action.author() == &review.subject_agent {
                return Ok(ValidateCallbackResult::Invalid(
                    "Agents cannot review themselves".into(),
                ));
            }

            if let Err(reason) = validate_contextual_review(&review) {
                return Ok(ValidateCallbackResult::Invalid(reason));
            }

            let evidence_result = validate_evidence(&review)?;

            if evidence_result != ValidateCallbackResult::Valid {
                return Ok(evidence_result);
            }

            validate_review_history(action.author(), action.prev_action().cloned(), &review)
        }

        FlatOp::Update(OpUpdate::Entry {
            app_entry: EntryTypes::ContextualReview(_),
            ..
        }) => Ok(ValidateCallbackResult::Invalid(
            "Published contextual reviews cannot be edited".into(),
        )),

        FlatOp::Delete(OpDelete { action }) => {
            let original = must_get_valid_record(action.deletes_address.clone())?;

            let review = original
                .entry()
                .to_app_option::<ContextualReview>()
                .map_err(|error| wasm_error!(WasmErrorInner::Guest(error.to_string())))?;

            if review.is_some() {
                return Ok(ValidateCallbackResult::Invalid(
                    "Published contextual reviews cannot be deleted".into(),
                ));
            }

            Ok(ValidateCallbackResult::Valid)
        }

        FlatOp::Link(OpLink::CreateLink {
            link_type: LinkTypes::SubjectToContextualReview,
            action,
        }) => {
            let Some(subject) = action.base_address.clone().into_agent_pub_key() else {
                return Ok(ValidateCallbackResult::Invalid(
                    "Review link base must be an agent".into(),
                ));
            };

            let Some(review_hash) = action.target_address.clone().into_action_hash() else {
                return Ok(ValidateCallbackResult::Invalid(
                    "Review link target must be an action hash".into(),
                ));
            };

            let record = must_get_valid_record(review_hash)?;

            if record.action().author() != action.author() {
                return Ok(ValidateCallbackResult::Invalid(
                    "Review link author must match the review author".into(),
                ));
            }

            let review = record
                .entry()
                .to_app_option::<ContextualReview>()
                .map_err(|error| wasm_error!(WasmErrorInner::Guest(error.to_string())))?;

            let Some(review) = review else {
                return Ok(ValidateCallbackResult::Invalid(
                    "Review link target must contain a contextual review".into(),
                ));
            };

            if review.subject_agent != subject {
                return Ok(ValidateCallbackResult::Invalid(
                    "Review link base must match the reviewed subject".into(),
                ));
            }

            if subject == *action.author() {
                return Ok(ValidateCallbackResult::Invalid(
                    "Agents cannot publish reviews about themselves".into(),
                ));
            }

            Ok(ValidateCallbackResult::Valid)
        }

        FlatOp::Link(OpLink::DeleteLink {
            link_type: LinkTypes::SubjectToContextualReview,
            ..
        }) => Ok(ValidateCallbackResult::Invalid(
            "Published contextual review links cannot be deleted".into(),
        )),

        _ => Ok(ValidateCallbackResult::Valid),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_supported_rating_and_domain() {
        let review = ContextualReview {
            schema_version: 1,
            subject_agent: AgentPubKey::from_raw_36(vec![0; 36]),
            domain: ReputationDomain::Commerce,
            rating: 5,
            comment: Some("Completed successfully".into()),
            evidence_attestations: vec![ActionHash::from_raw_36(vec![1; 36])],
        };

        assert!(validate_contextual_review(&review).is_ok());
        assert_eq!(review.domain.as_str(), "commerce");
    }

    #[test]
    fn rejects_rating_outside_supported_range() {
        let review = ContextualReview {
            schema_version: 1,
            subject_agent: AgentPubKey::from_raw_36(vec![0; 36]),
            domain: ReputationDomain::Technical,
            rating: 6,
            comment: None,
            evidence_attestations: vec![ActionHash::from_raw_36(vec![1; 36])],
        };

        assert!(validate_contextual_review(&review).is_err());
    }

    #[test]
    fn rejects_reviews_without_evidence() {
        let review = ContextualReview {
            schema_version: 1,
            subject_agent: AgentPubKey::from_raw_36(vec![0; 36]),
            domain: ReputationDomain::Community,
            rating: 4,
            comment: None,
            evidence_attestations: vec![],
        };

        assert!(validate_contextual_review(&review).is_err());
    }

    #[test]
    fn rejects_unsupported_schema_version() {
        let review = ContextualReview {
            schema_version: 2,
            subject_agent: AgentPubKey::from_raw_36(vec![0; 36]),
            domain: ReputationDomain::Reliability,
            rating: 4,
            comment: None,
            evidence_attestations: vec![ActionHash::from_raw_36(vec![1; 36])],
        };

        assert!(validate_contextual_review(&review).is_err());
    }
}
