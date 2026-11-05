
use hdi::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum AgentType {
    Human,
    #[serde(rename = "AI")]
    Ai,
    Service,
}

#[hdk_entry_helper]
pub struct AgentProfile {
    pub schema_version: u8,
    pub display_name: String,
    pub agent_type: AgentType,
    pub description: Option<String>,
}

#[hdk_entry_helper]
pub struct ReputationReview {
    pub schema_version: u8,
    pub subject_agent: AgentPubKey,
    pub rating: u8,
    pub comment: Option<String>,
}

#[hdk_entry_types]
#[unit_enum(UnitEntryTypes)]
pub enum EntryTypes {
    AgentProfile(AgentProfile),
    ReputationReview(ReputationReview),
}

#[hdk_link_types]
pub enum LinkTypes {
    AgentToProfile,
    AgentToReview,
}

pub fn validate_profile(profile: &AgentProfile) -> Result<(), String> {
    if profile.schema_version != 1 {
        return Err("Unsupported profile schema version".into());
    }

    let name = profile.display_name.trim();

    if name.is_empty() || name.chars().count() > 64 {
        return Err("Display name must be 1-64 characters".into());
    }

    if name.chars().any(char::is_control) {
        return Err("Display name cannot contain control characters".into());
    }

    if let Some(description) = &profile.description {
        if description.chars().count() > 280 {
            return Err("Description cannot exceed 280 characters".into());
        }

        if description.chars().any(char::is_control) {
            return Err("Description contains invalid control characters".into());
        }
    }

    Ok(())
}

pub fn validate_review(review: &ReputationReview) -> Result<(), String> {
    if review.schema_version != 1 {
        return Err("Unsupported review schema version".into());
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

    Ok(())
}

fn validate_review_history(
    author: &AgentPubKey,
    previous_action: Option<ActionHash>,
    subject_agent: &AgentPubKey,
) -> ExternResult<ValidateCallbackResult> {
    let Some(chain_top) = previous_action else {
        return Ok(ValidateCallbackResult::Invalid(
            "A reputation review must follow an existing source-chain action".into(),
        ));
    };

    let review_entry_type = EntryType::App(UnitEntryTypes::ReputationReview.try_into()?);

    let previous_activity = must_get_agent_activity(author.clone(), ChainFilter::new(chain_top))?;

    for activity in previous_activity {
        if activity.action.hashed.content.entry_type() != Some(&review_entry_type) {
            continue;
        }

        let previous_record = must_get_valid_record(activity.action.hashed.hash.clone())?;

        let previous_review = previous_record
            .entry()
            .to_app_option::<ReputationReview>()
            .map_err(|e| wasm_error!(WasmErrorInner::Guest(e.to_string())))?;

        let Some(previous_review) = previous_review else {
            continue;
        };

        if previous_review.subject_agent == *subject_agent {
            return Ok(ValidateCallbackResult::Invalid(
                "An agent can publish only one reputation review per subject".into(),
            ));
        }
    }

    Ok(ValidateCallbackResult::Valid)
}

/// Rejects a second profile link to the same target on the same source chain.
/// A profile update remains valid because it points to a new action hash.
fn validate_profile_link_history(
    author: &AgentPubKey,
    previous_action: Option<ActionHash>,
    candidate: &CreateLink,
) -> ExternResult<ValidateCallbackResult> {
    let Some(chain_top) = previous_action else {
        return Ok(ValidateCallbackResult::Invalid(
            "A profile link must follow an existing source-chain action".into(),
        ));
    };

    let previous_activity = must_get_agent_activity(author.clone(), ChainFilter::new(chain_top))?;

    for activity in previous_activity {
        if let Action::CreateLink(previous_link) = &activity.action.hashed.content {
            if previous_link.base_address == candidate.base_address
                && previous_link.target_address == candidate.target_address
                && previous_link.link_type == candidate.link_type
            {
                return Ok(ValidateCallbackResult::Invalid(
                    "An agent cannot create duplicate links to the same profile".into(),
                ));
            }
        }
    }

    Ok(ValidateCallbackResult::Valid)
}

fn validate_profile_update(
    update_author: &AgentPubKey,
    original_action_hash: ActionHash,
    profile: &AgentProfile,
) -> ExternResult<ValidateCallbackResult> {
    let original_record = must_get_valid_record(original_action_hash)?;

    if update_author != original_record.action().author() {
        return Ok(ValidateCallbackResult::Invalid(
            "Agents can only update their own profiles".into(),
        ));
    }

    if let Err(reason) = validate_profile(profile) {
        return Ok(ValidateCallbackResult::Invalid(reason));
    }

    Ok(ValidateCallbackResult::Valid)
}

#[hdk_extern]
pub fn validate(op: Op) -> ExternResult<ValidateCallbackResult> {
    match op.flattened::<EntryTypes, LinkTypes>()? {
        FlatOp::CreateEntry(OpEntry::CreateEntry {
            app_entry: EntryTypes::AgentProfile(profile),
            ..
        }) => match validate_profile(&profile) {
            Ok(()) => Ok(ValidateCallbackResult::Valid),
            Err(reason) => Ok(ValidateCallbackResult::Invalid(reason)),
        },

        FlatOp::CreateEntry(OpEntry::CreateEntry {
            app_entry: EntryTypes::ReputationReview(review),
            action,
        }) => {
            if action.author() == &review.subject_agent {
                return Ok(ValidateCallbackResult::Invalid(
                    "Agents cannot review themselves".into(),
                ));
            }

            if let Err(reason) = validate_review(&review) {
                return Ok(ValidateCallbackResult::Invalid(reason));
            }

            validate_review_history(
                action.author(),
                action.prev_action().cloned(),
                &review.subject_agent,
            )
        }

        FlatOp::Update(OpUpdate::Entry {
            app_entry: EntryTypes::AgentProfile(profile),
            action,
        }) => validate_profile_update(
            action.author(),
            action.original_action_address.clone(),
            &profile,
        ),

        FlatOp::Update(OpUpdate::Entry {
            app_entry: EntryTypes::ReputationReview(_),
            ..
        }) => Ok(ValidateCallbackResult::Invalid(
            "Published reputation reviews cannot be edited".into(),
        )),

        FlatOp::Link(OpLink::CreateLink {
            link_type: LinkTypes::AgentToProfile,
            action,
        }) => {
            let Some(owner) = action.base_address.clone().into_agent_pub_key() else {
                return Ok(ValidateCallbackResult::Invalid(
                    "Profile link base must be an agent".into(),
                ));
            };

            if &owner != action.author() {
                return Ok(ValidateCallbackResult::Invalid(
                    "Profile links must be created by their owning agent".into(),
                ));
            }

            let Some(target) = action.target_address.clone().into_action_hash() else {
                return Ok(ValidateCallbackResult::Invalid(
                    "Profile link target must be an action hash".into(),
                ));
            };

            let duplicate_check = validate_profile_link_history(
                action.author(),
                action.prev_action().cloned(),
                &action,
            )?;

            if duplicate_check != ValidateCallbackResult::Valid {
                return Ok(duplicate_check);
            }

            let record = must_get_valid_record(target)?;

            if record.action().author() != &owner {
                return Ok(ValidateCallbackResult::Invalid(
                    "Profile link owner must match profile author".into(),
                ));
            }

            let profile = record
                .entry()
                .to_app_option::<AgentProfile>()
                .map_err(|e| wasm_error!(WasmErrorInner::Guest(e.to_string())))?;

            if profile.is_none() {
                return Ok(ValidateCallbackResult::Invalid(
                    "Profile link target must contain an agent profile".into(),
                ));
            }

            Ok(ValidateCallbackResult::Valid)
        }

        FlatOp::Link(OpLink::CreateLink {
            link_type: LinkTypes::AgentToReview,
            action,
        }) => {
            let Some(owner) = action.base_address.clone().into_agent_pub_key() else {
                return Ok(ValidateCallbackResult::Invalid(
                    "Review link base must be an agent".into(),
                ));
            };

            let Some(target) = action.target_address.clone().into_action_hash() else {
                return Ok(ValidateCallbackResult::Invalid(
                    "Review link target must be an action hash".into(),
                ));
            };

            let record = must_get_valid_record(target)?;

            if record.action().author() != action.author() {
                return Ok(ValidateCallbackResult::Invalid(
                    "Review link creator must be the review author".into(),
                ));
            }

            let review = record
                .entry()
                .to_app_option::<ReputationReview>()
                .map_err(|e| wasm_error!(WasmErrorInner::Guest(e.to_string())))?;

            let Some(review) = review else {
                return Ok(ValidateCallbackResult::Invalid(
                    "Review link target must contain a reputation review".into(),
                ));
            };

            if review.subject_agent != owner {
                return Ok(ValidateCallbackResult::Invalid(
                    "Review link base must match the review subject".into(),
                ));
            }

            if review.subject_agent == *action.author() {
                return Ok(ValidateCallbackResult::Invalid(
                    "Agents cannot publish reviews about themselves".into(),
                ));
            }

            Ok(ValidateCallbackResult::Valid)
        }

        FlatOp::Link(OpLink::DeleteLink {
            link_type: LinkTypes::AgentToReview,
            ..
        }) => Ok(ValidateCallbackResult::Invalid(
            "Reputation review links cannot be deleted".into(),
        )),

        FlatOp::Link(OpLink::DeleteLink {
            original_action,
            action,
            ..
        }) => {
            if action.author() != original_action.author() {
                return Ok(ValidateCallbackResult::Invalid(
                    "Only the link author can delete a link".into(),
                ));
            }

            Ok(ValidateCallbackResult::Valid)
        }

        FlatOp::Delete(OpDelete { action }) => {
            let original_record = must_get_valid_record(action.deletes_address.clone())?;

            let review = original_record
                .entry()
                .to_app_option::<ReputationReview>()
                .map_err(|e| wasm_error!(WasmErrorInner::Guest(e.to_string())))?;

            if review.is_some() {
                return Ok(ValidateCallbackResult::Invalid(
                    "Published reputation reviews cannot be deleted".into(),
                ));
            }

            if action.author() != original_record.action().author() {
                return Ok(ValidateCallbackResult::Invalid(
                    "Only the author can delete a record".into(),
                ));
            }

            Ok(ValidateCallbackResult::Valid)
        }

        _ => Ok(ValidateCallbackResult::Valid),
    }
}
