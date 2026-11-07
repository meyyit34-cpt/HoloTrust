use holochain::prelude::{ActionHash, AgentPubKey, Timestamp};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ClaimType {
    Skill,
    Experience,
    ServiceCompletion,
    Membership,
    Credential,
    Other,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum EvidenceReference {
    Action(ActionHash),
    Entry(holochain::prelude::EntryHash),
    External { url: String, sha256: String },
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateAttestationInput {
    pub subject: AgentPubKey,
    pub claim_type: ClaimType,
    pub claim: String,
    pub evidence: Vec<EvidenceReference>,
    pub expires_at: Option<Timestamp>,
    pub supersedes: Option<ActionHash>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetAttestationInput {
    pub action_hash: ActionHash,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum AttestationStatus {
    Active,
    Revoked,
    Expired,
    Superseded,
}
