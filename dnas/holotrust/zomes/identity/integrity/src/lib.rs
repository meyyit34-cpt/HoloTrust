use hdi::prelude::*;
use serde::{Deserialize, Serialize};
/// The declared category of a HoloTrust participant.
///
/// This is a self-declared category, not a verified real-world identity.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum AgentType {
    Human,
    #[serde(rename = "AI")]
    Ai,
    Service,
}
/// Public profile associated with its publishing Holochain agent.
#[hdk_entry_helper]
pub struct AgentProfile {
    pub schema_version: u8,
    pub display_name: String,
    pub agent_type: AgentType,
    pub description: Option<String>,
}
/// All entry types defined by the Identity integrity zome.
#[hdk_entry_types]
#[unit_enum(UnitEntryTypes)]
pub enum EntryTypes {
    AgentProfile(AgentProfile),
}
/// Validate the application-level constraints of a profile.
///
/// Character limits count Unicode scalar values, not UTF-8 bytes.
pub fn validate_profile(profile: &AgentProfile) -> Result<(), String> {
    if profile.schema_version != 1 {
        return Err("Unsupported profile schema version".to_string());
    }
    let display_name = profile.display_name.trim();
    if display_name.is_empty() {
        return Err("Display name cannot be empty".to_string());
    }
    if display_name.chars().count() > 64 {
        return Err("Display name cannot exceed 64 characters".to_string());
    }
    if display_name.chars().any(char::is_control) {
        return Err("Display name cannot contain control characters".to_string());
    }
    if let Some(description) = &profile.description {
        if description.chars().count() > 280 {
            return Err("Description cannot exceed 280 characters".to_string());
        }
    }
    Ok(())
}
/// Holochain validation callback.
///
/// Every peer independently validates profile entries before accepting
/// them into the DHT.
#[hdk_extern]
pub fn validate(op: Op) -> ExternResult<ValidateCallbackResult> {
    match op.flattened()? {
        FlatOp::CreateEntry(OpEntry::CreateEntry {
            app_entry: EntryTypes::AgentProfile(profile),
            ..
        }) => {
            match validate_profile(&profile) {
                Ok(()) => Ok(ValidateCallbackResult::Valid),
                Err(reason) => Ok(ValidateCallbackResult::Invalid(reason)),
            }
        }
        _ => Ok(ValidateCallbackResult::Valid),
    }
}
