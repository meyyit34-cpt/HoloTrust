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

            match validate_review(&review) {
                Ok(()) => Ok(ValidateCallbackResult::Valid),
                Err(reason) => Ok(ValidateCallbackResult::Invalid(reason)),
            }
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
            let owner = action.base_address.clone().into_agent_pub_key();
            let target = action.target_address.clone().into_action_hash();

            if owner.as_ref() == Some(action.author()) && target.is_some() {
                Ok(ValidateCallbackResult::Valid)
            } else {
                Ok(ValidateCallbackResult::Invalid(
                    "Profile links must be created by their owning agent".into(),
                ))
            }
        }

        FlatOp::Link(OpLink::CreateLink {
            link_type: LinkTypes::AgentToReview,
            action,
        }) => {
            let subject = action.base_address.clone().into_agent_pub_key();

            let Some(subject) = subject else {
                return Ok(ValidateCallbackResult::Invalid(
                    "Review link base must be an agent".into(),
                ));
            };

            if &subject == action.author() {
                return Ok(ValidateCallbackResult::Invalid(
                    "Agents cannot link reviews to themselves".into(),
                ));
            }

            let Some(target) = action.target_address.clone().into_action_hash() else {
                return Ok(ValidateCallbackResult::Invalid(
                    "Review link target must be an action hash".into(),
                ));
            };

            let record = must_get_valid_record(target)?;

            if record.action().author() != action.author() {
                return Ok(ValidateCallbackResult::Invalid(
                    "Review link author must match review author".into(),
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

            if review.subject_agent != subject {
                return Ok(ValidateCallbackResult::Invalid(
                    "Review subject must match the link base agent".into(),
                ));
            }

            Ok(ValidateCallbackResult::Valid)
        }

        _ => Ok(ValidateCallbackResult::Valid),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_profile() -> AgentProfile {
        AgentProfile {
            schema_version: 1,
            display_name: "HoloTrust User".into(),
            agent_type: AgentType::Human,
            description: Some("A test profile".into()),
        }
    }

    #[test]
    fn accepts_valid_profile() {
        assert!(validate_profile(&valid_profile()).is_ok());
    }

    #[test]
    fn rejects_empty_name() {
        let mut profile = valid_profile();
        profile.display_name = "  ".into();
        assert!(validate_profile(&profile).is_err());
    }

    #[test]
    fn rejects_long_name() {
        let mut profile = valid_profile();
        profile.display_name = "a".repeat(65);
        assert!(validate_profile(&profile).is_err());
    }

    #[test]
    fn accepts_unicode_name() {
        let mut profile = valid_profile();
        profile.display_name = "Mehmet 世界".into();
        assert!(validate_profile(&profile).is_ok());
    }

    #[test]
    fn rejects_control_characters() {
        let mut profile = valid_profile();
        profile.display_name = "HoloTrust\nUser".into();
        assert!(validate_profile(&profile).is_err());
    }

    #[test]
    fn rejects_long_description() {
        let mut profile = valid_profile();
        profile.description = Some("a".repeat(281));
        assert!(validate_profile(&profile).is_err());
    }

    #[test]
    fn rejects_control_characters_in_description() {
        let mut profile = valid_profile();
        profile.description = Some("bad\ntext".into());
        assert!(validate_profile(&profile).is_err());
    }

    #[test]
    fn rejects_wrong_profile_schema() {
        let mut profile = valid_profile();
        profile.schema_version = 2;
        assert!(validate_profile(&profile).is_err());
    }

    fn valid_review() -> ReputationReview {
        ReputationReview {
            schema_version: 1,
            subject_agent: AgentPubKey::from_raw_36(vec![0; 36]),
            rating: 5,
            comment: Some("Reliable agent".into()),
        }
    }

    #[test]
    fn accepts_valid_review() {
        assert!(validate_review(&valid_review()).is_ok());
    }

    #[test]
    fn rejects_zero_rating() {
        let mut review = valid_review();
        review.rating = 0;
        assert!(validate_review(&review).is_err());
    }

    #[test]
    fn rejects_rating_above_five() {
        let mut review = valid_review();
        review.rating = 6;
        assert!(validate_review(&review).is_err());
    }

    #[test]
    fn rejects_long_review() {
        let mut review = valid_review();
        review.comment = Some("a".repeat(501));
        assert!(validate_review(&review).is_err());
    }

    #[test]
    fn accepts_review_without_comment() {
        let mut review = valid_review();
        review.comment = None;
        assert!(validate_review(&review).is_ok());
    }
}
