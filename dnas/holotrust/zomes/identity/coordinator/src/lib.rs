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

        let Some(record) = get(action_hash, GetOptions::default())? else {
            continue;
        };

        if record.action().author() != &input.agent_pub_key {
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
pub fn update_profile(input: UpdateProfileInput) -> ExternResult<ActionHash> {
    let agent = agent_info()?.agent_initial_pubkey;

    let original_record = get(input.action_hash.clone(), GetOptions::default())?.ok_or(
        wasm_error!(WasmErrorInner::Guest("Profile not found".into())),
    )?;

    if original_record.action().author() != &agent
        || original_record
            .entry()
            .to_app_option::<AgentProfile>()
            .map_err(|e| wasm_error!(WasmErrorInner::Guest(e.to_string())))?
            .is_none()
    {
        return Err(wasm_error!(WasmErrorInner::Guest(
            "You can only update your own profile".into()
        )));
    }

    let is_latest = match get_my_profile(())? {
        Some(record) => record.action_address() == &input.action_hash,
        None => false,
    };

    if !is_latest {
        return Err(wasm_error!(WasmErrorInner::Guest(
            "Profile updates must target your latest profile version".into()
        )));
    }

    let profile = profile_from_input(input.display_name, input.agent_type, input.description);

    validate_profile(&profile).map_err(|reason| wasm_error!(WasmErrorInner::Guest(reason)))?;

    let old_hash = input.action_hash.clone();
    let updated_hash = update_entry(input.action_hash, &EntryTypes::AgentProfile(profile))?;

    let links = get_links(
        LinkQuery::try_new(agent.clone(), LinkTypes::AgentToProfile)?,
        GetStrategy::default(),
    )?;

    for link in links {
        if link.author != agent {
            continue;
        }

        let Some(target_hash) = link.target.clone().into_action_hash() else {
            continue;
        };

        if target_hash != old_hash {
            continue;
        }

        delete_link(link.create_link_hash, GetOptions::default())?;
    }

    index_profile(updated_hash.clone())?;

    Ok(updated_hash)
}

#[hdk_extern]
pub fn create_review(input: CreateReviewInput) -> ExternResult<ActionHash> {
    let author = agent_info()?.agent_initial_pubkey;

    if author == input.subject_agent {
        return Err(wasm_error!(WasmErrorInner::Guest(
            "You cannot review your own agent".into()
        )));
    }

    let review = ReputationReview {
        schema_version: 1,
        subject_agent: input.subject_agent.clone(),
        rating: input.rating,
        comment: input.comment,
    };

    validate_review(&review).map_err(|reason| wasm_error!(WasmErrorInner::Guest(reason)))?;

    let existing_links = get_links(
        LinkQuery::try_new(input.subject_agent.clone(), LinkTypes::AgentToReview)?,
        GetStrategy::default(),
    )?;

    for link in existing_links {
        if link.author != author {
            continue;
        }

        let Some(existing_hash) = link.target.into_action_hash() else {
            continue;
        };

        let Some(existing_record) = get(existing_hash, GetOptions::default())? else {
            continue;
        };

        if existing_record.action().author() != &author {
            continue;
        }

        let existing_review = match existing_record.entry().to_app_option::<ReputationReview>() {
            Ok(Some(review)) => review,
            _ => continue,
        };

        if existing_review.subject_agent == input.subject_agent
            && validate_review(&existing_review).is_ok()
        {
            return Err(wasm_error!(WasmErrorInner::Guest(
                "You have already reviewed this agent".into()
            )));
        }
    }

    let review_hash = create_entry(EntryTypes::ReputationReview(review))?;

    create_link(
        input.subject_agent,
        review_hash.clone(),
        LinkTypes::AgentToReview,
        "review",
    )?;

    Ok(review_hash)
}

#[hdk_extern]
pub fn get_reviews(input: GetReviewsInput) -> ExternResult<Vec<Record>> {
    let links = get_links(
        LinkQuery::try_new(input.subject_agent.clone(), LinkTypes::AgentToReview)?,
        GetStrategy::default(),
    )?;

    let mut latest_by_author: HashMap<AgentPubKey, Record> = HashMap::new();

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

        let review = match record.entry().to_app_option::<ReputationReview>() {
            Ok(Some(review)) => review,
            _ => continue,
        };

        if review.subject_agent != input.subject_agent {
            continue;
        }

        if validate_review(&review).is_err() {
            continue;
        }

        let author = record.action().author().clone();

        let is_newer = match latest_by_author.get(&author) {
            Some(existing) => record.action().timestamp() > existing.action().timestamp(),
            None => true,
        };

        if is_newer {
            latest_by_author.insert(author, record);
        }
    }

    let mut records: Vec<Record> = latest_by_author.into_values().collect();
    records.sort_by_key(|record| record.action().timestamp());

    Ok(records)
}

#[hdk_extern]
pub fn get_reputation_summary(input: GetReviewsInput) -> ExternResult<ReputationSummary> {
    let records = get_reviews(input)?;

    let mut total_rating: u64 = 0;
    let mut review_count: u32 = 0;

    for record in records {
        let review = match record.entry().to_app_option::<ReputationReview>() {
            Ok(Some(review)) => review,
            _ => continue,
        };

        if validate_review(&review).is_err() {
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

    Ok(ReputationSummary {
        review_count,
        average_rating,
    })
}

// END OF FILE
