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

#[hdk_entry_types]
#[unit_enum(UnitEntryTypes)]
pub enum EntryTypes {
    AgentProfile(AgentProfile),
}

#[hdk_link_types]
pub enum LinkTypes {
    AgentToProfile,
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
    }

    Ok(())
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
                    "Profile links must be created by their owning agent and target an action"
                        .into(),
                ))
            }
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
        let mut p = valid_profile();
        p.display_name = "  ".into();
        assert!(validate_profile(&p).is_err());
    }

    #[test]
    fn rejects_long_name() {
        let mut p = valid_profile();
        p.display_name = "a".repeat(65);
        assert!(validate_profile(&p).is_err());
    }

    #[test]
    fn accepts_name_at_64_character_limit() {
        let mut p = valid_profile();
        p.display_name = "a".repeat(64);
        assert!(validate_profile(&p).is_ok());
    }

    #[test]
    fn rejects_control_characters_in_name() {
        let mut p = valid_profile();
        p.display_name = "HoloTrust\nUser".into();
        assert!(validate_profile(&p).is_err());
    }

    #[test]
    fn accepts_unicode_name() {
        let mut p = valid_profile();
        p.display_name = "Mehmet 世界".into();
        assert!(validate_profile(&p).is_ok());
    }

    #[test]
    fn rejects_long_description() {
        let mut p = valid_profile();
        p.description = Some("a".repeat(281));
        assert!(validate_profile(&p).is_err());
    }

    #[test]
    fn accepts_description_at_280_character_limit() {
        let mut p = valid_profile();
        p.description = Some("a".repeat(280));
        assert!(validate_profile(&p).is_ok());
    }

    #[test]
    fn accepts_optional_empty_description() {
        let mut p = valid_profile();
        p.description = None;
        assert!(validate_profile(&p).is_ok());
    }

    #[test]
    fn rejects_wrong_schema() {
        let mut p = valid_profile();
        p.schema_version = 2;
        assert!(validate_profile(&p).is_err());
    }
}
