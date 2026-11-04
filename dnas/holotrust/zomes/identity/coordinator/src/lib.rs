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

fn index_profile(action_hash: ActionHash) -> ExternResult
