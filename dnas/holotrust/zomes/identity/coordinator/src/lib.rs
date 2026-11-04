use hdk::prelude::*;
use identity_integrity::{
    validate_profile, validate_review, AgentProfile, AgentType, EntryTypes, LinkTypes,
    ReputationReview,
};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateProfileInput {
    pub display_name: String,
    pub agent_type: AgentType,
    pub description: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetProfileInput {
    pub agent_pub_key: AgentPubKey,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateProfileInput {
    pub action_hash: ActionHash,
    pub display_name: String,
    pub agent_type: AgentType,
    pub description: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateReviewInput {
    pub subject_agent: AgentPubKey,
    pub rating: u8,
    pub comment: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetReviewsInput {
    pub subject_agent: AgentPubKey,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ReputationSummary {
    pub review_count: u32,
    pub average_rating: f64,
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
    if get_my_profile(())?.is_some() {
        return Err(wasm_error!(WasmErrorInner::Guest(
            "This agent already has a profile. Use update_profile instead.".into()
        )));
    }

    let profile = profile_from_input(input.display_name, input.agent_type, input.description);

    let action_hash = create_entry(EntryTypes::AgentProfile(profile))?;
    index_profile(action_hash.clone())?;

    Ok(action_hash)
}

#[hdk_extern]
pub fn get_my_profile(_: ()) -> ExternResult<Option<Record>> {
    let agent = agent_info()?.agent_initial_pubkey;

    let links = get_links(
        LinkQuery::try_new(agent.clone(), LinkTypes::AgentToProfile)?,
        GetStrategy::default(),
    )?;

    for link in links.into_iter().rev() {
        if link.author != agent {
            continue;
        }

        let Some(action_hash) = link.target.into_action_hash() else {
            continue;
        };

        let Some(record) = get(action_hash, GetOptions::default())? else {
            continue;
        };

        if record.action().author() != &agent {
            continue;
        }

        let profile = match record.entry().to_app_option::<AgentProfile>() {
            Ok(Some(profile)) => profile,
            _ => continue,
        };

        if validate_profile(&profile).is_err() {
            continue;
        }

        return Ok(Some(record));
    }

    Ok(None)
}

#[hdk_extern]
pub fn get_profile(input: GetProfileInput) -> ExternResult<Option<Record>> {
    let links = get_links(
        LinkQuery::try_new(input.agent_pub_key.clone(), LinkTypes::AgentToProfile)?,
        GetStrategy::default(),
    )?;

    for link in links.into_iter().rev() {
        if link.author != input.agent_pub_key {
            continue;
        }

        let Some(action_hash) = link.target.into_action_hash() else {
            continue;
        };

        let Some(record) = get(action_hash
