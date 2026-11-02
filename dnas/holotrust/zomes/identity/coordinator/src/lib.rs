
use hdk::prelude::*;
use identity_integrity::{
    AgentProfile, AgentType, EntryTypes, LinkTypes, ReputationReview,
};

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

    create_link(
        agent,
        action_hash,
        LinkTypes::AgentToProfile,
        "profile",
    )?;

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
    let agent = agent_info()?.agent_initial_pubkey;

    let original_record = get(
        input.action_hash.clone(),
        GetOptions::default(),
    )?
    .ok_or(wasm_error!(WasmErrorInner::Guest(
        "Profile not found".into(),
    )))?;

    if original_record.action().author() != &agent {
        return Err(wasm_error!(WasmErrorInner::Guest(
            "You can only update your own profile".into(),
        )));
    }

    let profile = profile_from_input(
        input.display_name,
        input.agent_type,
        input.description,
    );

    let updated_hash = update_entry(
        input.action_hash,
        &EntryTypes::AgentProfile(profile),
    )?;

    let links = get_links(
        LinkQuery::try_new(agent, LinkTypes::AgentToProfile)?,
        GetStrategy::default(),
    )?;

    for link in links {
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
            "You cannot review your own agent".into(),
        )));
    }

    let review = ReputationReview {
        schema_version: 1,
        subject_agent: input.subject_agent.clone(),
        rating: input.rating,
        comment: input.comment,
    };

    let review_hash = create_entry(
        EntryTypes::ReputationReview(review),
    )?;

    create_link(
        input.subject_agent,
        review_hash.clone(),
        LinkTypes::AgentToReview,
        "review",
    )?;

    Ok(review_hash)
}

#[hdk_extern]
pub fn get_reviews(
    input: GetReviewsInput,
) -> ExternResult<Vec<Record>> {
    let links = get_links(
        LinkQuery::try_new(
            input.subject_agent,
            LinkTypes::AgentToReview,
        )?,
        GetStrategy::default(),
    )?;

    let mut records = Vec::new();

    for link in links {
        if let Some(action_hash) = link.target.into_action_hash() {
            if let Some(record) = get(
                action_hash,
                GetOptions::default(),
            )? {
                records.push(record);
            }
        }
    }

    Ok(records)
}

#[hdk_extern]
pub fn get_reputation_summary(
    input: GetReviewsInput,
) -> ExternResult<ReputationSummary> {
    let records = get_reviews(input)?;

    let mut total_rating: u64 = 0;
    let mut review_count: u32 = 0;

    for record in records {
        if let Some(review) = record
            .entry()
            .to_app_option::<ReputationReview>()?
        {
            total_rating += u64::from(review.rating);
            review_count += 1;
        }
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
