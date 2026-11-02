use hdk::prelude::*;
use identity_integrity::{AgentProfile, AgentType, EntryTypes, LinkTypes};

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateProfileInput {
    pub display_name: String,
    pub agent_type: AgentType,
    pub description: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetProfileInput {
    pub action_hash: ActionHash,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateProfileInput {
    pub action_hash: ActionHash,
    pub display_name: String,
    pub agent_type: AgentType,
    pub description: Option<String>,
}

fn profile_from_input(
    display_name: String,
    agent_type: AgentType,
    description: Option<String>,
) -> AgentProfile {
    AgentProfile {
        schema_version: 1,
        display_name,
        agent_type,
        description,
    }
}

fn index_profile(action_hash: ActionHash) -> ExternResult<()> {
    let agent = agent_info()?.agent_initial_pubkey;
    create_link(agent, action_hash, LinkTypes::AgentToProfile, "profile")?;
    Ok(())
}

#[hdk_extern]
pub fn create_profile(input: CreateProfileInput) -> ExternResult<ActionHash> {
    let profile = profile_from_input(
        input.display_name,
        input.agent_type,
        input.description,
    );
    let action_hash = create_entry(EntryTypes::AgentProfile(profile))?;
    index_profile(action_hash.clone())?;
    Ok(action_hash)
}

#[hdk_extern]
pub fn get_my_profile(_: ()) -> ExternResult<Option<Record>> {
    let agent = agent_info()?.agent_initial_pubkey;
    let links = get_links(
        LinkQuery::try_new(agent, LinkTypes::AgentToProfile)?,
        GetStrategy::default(),
    )?;

    let Some(link) = links.into_iter().last() else {
        return Ok(None);
    };

    let Some(action_hash) = link.target.into_action_hash() else {
        return Ok(None);
    };

    get(action_hash, GetOptions::default())
}

#[hdk_extern]
pub fn get_profile(input: GetProfileInput) -> ExternResult<Option<Record>> {
    get(input.action_hash, GetOptions::default())
}

#[hdk_extern]
pub fn update_profile(input: UpdateProfileInput) -> ExternResult<ActionHash> {
    let profile = profile_from_input(
        input.display_name,
        input.agent_type,
        input.description,
    );
    let updated_hash = update_entry(input.action_hash, &EntryTypes::AgentProfile(profile))?;

    let agent = agent_info()?.agent_initial_pubkey;
    let links = get_links(
        LinkQuery::try_new(agent.clone(), LinkTypes::AgentToProfile)?,
        GetStrategy::default(),
    )?;

    for link in links {
        delete_link(link.create_link_hash)?;
    }

    index_profile(updated_hash.clone())?;
    Ok(updated_hash)
}
