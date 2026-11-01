use hdk::prelude::*;
use identity_integrity::{AgentProfile, AgentType, EntryTypes};

/// Input required to create a public HoloTrust profile.
#[derive(Serialize, Deserialize, Debug)]
pub struct CreateProfileInput {
    pub display_name: String,
    pub agent_type: AgentType,
    pub description: Option<String>,
}

/// Input used to retrieve a profile by its action hash.
#[derive(Serialize, Deserialize, Debug)]
pub struct GetProfileInput {
    pub action_hash: ActionHash,
}

/// Creates a profile and publishes it to the Holochain DHT.
#[hdk_extern]
pub fn create_profile(input: CreateProfileInput) -> ExternResult<ActionHash> {
    let profile = AgentProfile {
        schema_version: 1,
        display_name: input.display_name,
        agent_type: input.agent_type,
        description: input.description,
    };

    let action_hash = create_entry(EntryTypes::AgentProfile(profile))?;

    Ok(action_hash)
}

/// Retrieves a previously published profile by its action hash.
#[hdk_extern]
pub fn get_profile(input: GetProfileInput) -> ExternResult<Option<Record>> {
    get(input.action_hash, GetOptions::default())
}
