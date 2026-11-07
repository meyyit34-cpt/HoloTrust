use hdi::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrustContext {
    General,
    Commerce,
    Professional,
    Hospitality,
    Technical,
    Community,
    Reliability,
}

#[hdk_entry_helper]
pub struct TrustRelationship {
    pub schema_version: u8,
    pub subject_agent: AgentPubKey,
    pub context: TrustContext,
}

#[hdk_entry_helper]
pub struct TrustRemoval {
    pub schema_version: u8,
    pub relationship_action: ActionHash,
}

#[hdk_entry_types]
#[unit_enum(UnitEntryTypes)]
pub enum EntryTypes {
    TrustRelationship(TrustRelationship),
    TrustRemoval(TrustRemoval),
}

#[hdk_link_types]
pub enum LinkTypes {
    AuthorToTrustRelationship,
    SubjectToTrustRelationship,
    RelationshipToRemoval,
}

pub fn validate_trust_relationship(
    relationship: &TrustRelationship,
    author: &AgentPubKey,
) -> Result<(), String> {
    if relationship.schema_version != 1 {
        return Err("Unsupported trust relationship schema version".into());
    }

    if &relationship.subject_agent == author {
        return Err("Agents cannot create trust relationships with themselves".into());
    }

    Ok(())
}

pub fn validate_trust_removal(removal: &TrustRemoval) -> Result<(), String> {
    if removal.schema_version != 1 {
        return Err("Unsupported trust removal schema version".into());
    }

    Ok(())
}

fn validate_no_duplicate_active_relationship(
    author: &AgentPubKey,
    previous_action: Option<ActionHash>,
    candidate: &TrustRelationship,
) -> ExternResult<ValidateCallbackResult> {
    let Some(chain_top) = previous_action else {
        return Ok(ValidateCallbackResult::Invalid(
            "A trust relationship must follow an existing source-chain action".into(),
        ));
    };

    let relationship_entry_type = EntryType::App(UnitEntryTypes::TrustRelationship.try_into()?);
    let removal_entry_type = EntryType::App(UnitEntryTypes::TrustRemoval.try_into()?);

    let activity = must_get_agent_activity(author.clone(), ChainFilter::new(chain_top))?;

    let mut removed_relationships: Vec<ActionHash> = Vec::new();

    for item in &activity {
        if item.action.hashed.content.entry_type() != Some(&removal_entry_type) {
            continue;
        }

        let record = must_get_valid_record(item.action.hashed.hash.clone())?;
        let removal = record
            .entry()
            .to_app_option::<TrustRemoval>()
            .map_err(|error| wasm_error!(WasmErrorInner::Guest(error.to_string())))?;

        if let Some(removal) = removal {
            removed_relationships.push(removal.relationship_action);
        }
    }

    for item in activity {
        if item.action.hashed.content.entry_type() != Some(&relationship_entry_type) {
            continue;
        }

        let relationship_hash = item.action.hashed.hash.clone();
        let record = must_get_valid_record(relationship_hash.clone())?;
        let relationship = record
            .entry()
            .to_app_option::<TrustRelationship>()
            .map_err(|error| wasm_error!(WasmErrorInner::Guest(error.to_string())))?;

        let Some(relationship) = relationship else {
            continue;
        };

        let same_relationship = relationship.subject_agent == candidate.subject_agent
            && relationship.context == candidate.context;

        let already_removed = removed_relationships
            .iter()
            .any(|hash| hash == &relationship_hash);

        if same_relationship && !already_removed {
            return Ok(ValidateCallbackResult::Invalid(
                "An agent cannot have duplicate active trust relationships with the same subject and context".into(),
            ));
        }
    }

    Ok(ValidateCallbackResult::Valid)
}

fn validate_no_duplicate_removal(
    author: &AgentPubKey,
    previous_action: Option<ActionHash>,
    candidate: &TrustRemoval,
) -> ExternResult<ValidateCallbackResult> {
    let Some(chain_top) = previous_action else {
        return Ok(ValidateCallbackResult::Invalid(
            "A trust removal must follow an existing source-chain action".into(),
        ));
    };

    let removal_entry_type = EntryType::App(UnitEntryTypes::TrustRemoval.try_into()?);
    let activity = must_get_agent_activity(author.clone(), ChainFilter::new(chain_top))?;

    for item in activity {
        if item.action.hashed.content.entry_type() != Some(&removal_entry_type) {
            continue;
        }

        let record = must_get_valid_record(item.action.hashed.hash.clone())?;
        let removal = record
            .entry()
            .to_app_option::<TrustRemoval>()
            .map_err(|error| wasm_error!(WasmErrorInner::Guest(error.to_string())))?;

        if let Some(removal) = removal {
            if removal.relationship_action == candidate.relationship_action {
                return Ok(ValidateCallbackResult::Invalid(
                    "A trust relationship can only be removed once".into(),
                ));
            }
        }
    }

    Ok(ValidateCallbackResult::Valid)
}

#[hdk_extern]
pub fn validate(op: Op) -> ExternResult<ValidateCallbackResult> {
    match op.flattened::<EntryTypes, LinkTypes>()? {
        FlatOp::CreateEntry(OpEntry::CreateEntry {
            app_entry: EntryTypes::TrustRelationship(relationship),
            action,
        }) => {
            if let Err(reason) = validate_trust_relationship(&relationship, action.author()) {
                return Ok(ValidateCallbackResult::Invalid(reason));
            }

            validate_no_duplicate_active_relationship(
                action.author(),
                action.prev_action().cloned(),
                &relationship,
            )
        }

        FlatOp::CreateEntry(OpEntry::CreateEntry {
            app_entry: EntryTypes::TrustRemoval(removal),
            action,
        }) => {
            if let Err(reason) = validate_trust_removal(&removal) {
                return Ok(ValidateCallbackResult::Invalid(reason));
            }

            let original = must_get_valid_record(removal.relationship_action.clone())?;
            let relationship = original
                .entry()
                .to_app_option::<TrustRelationship>()
                .map_err(|error| wasm_error!(WasmErrorInner::Guest(error.to_string())))?;

            if relationship.is_none() {
                return Ok(ValidateCallbackResult::Invalid(
                    "Trust removal target must be a trust relationship".into(),
                ));
            }

            if original.action().author() != action.author() {
                return Ok(ValidateCallbackResult::Invalid(
                    "Only the original author can remove a trust relationship".into(),
                ));
            }

            validate_no_duplicate_removal(
                action.author(),
                action.prev_action().cloned(),
                &removal,
            )
        }

        FlatOp::Link(OpLink::CreateLink {
            link_type: LinkTypes::AuthorToTrustRelationship,
            action,
        }) => {
            let Some(author) = action.base_address.clone().into_agent_pub_key() else {
                return Ok(ValidateCallbackResult::Invalid(
                    "Trust author link base must be an agent".into(),
                ));
            };

            if &author != action.author() {
                return Ok(ValidateCallbackResult::Invalid(
                    "Trust author link base must match its author".into(),
                ));
            }

            let Some(target) = action.target_address.clone().into_action_hash() else {
                return Ok(ValidateCallbackResult::Invalid(
                    "Trust author link target must be an action hash".into(),
                ));
            };

            let record = must_get_valid_record(target)?;

            if record.action().author() != &author {
                return Ok(ValidateCallbackResult::Invalid(
                    "Trust author link must reference a relationship created by its author".into(),
                ));
            }

            let relationship = record
                .entry()
                .to_app_option::<TrustRelationship>()
                .map_err(|error| wasm_error!(WasmErrorInner::Guest(error.to_string())))?;

            let Some(relationship) = relationship else {
                return Ok(ValidateCallbackResult::Invalid(
                    "Trust author link target must contain a trust relationship".into(),
                ));
            };

            if let Err(reason) = validate_trust_relationship(&relationship, &author) {
                return Ok(ValidateCallbackResult::Invalid(reason));
            }

            Ok(ValidateCallbackResult::Valid)
        }

        FlatOp::Link(OpLink::CreateLink {
            link_type: LinkTypes::SubjectToTrustRelationship,
            action,
        }) => {
            let Some(subject) = action.base_address.clone().into_agent_pub_key() else {
                return Ok(ValidateCallbackResult::Invalid(
                    "Trust subject link base must be an agent".into(),
                ));
            };

            let Some(target) = action.target_address.clone().into_action_hash() else {
                return Ok(ValidateCallbackResult::Invalid(
                    "Trust subject link target must be an action hash".into(),
                ));
            };

            let record = must_get_valid_record(target)?;
            let relationship = record
                .entry()
                .to_app_option::<TrustRelationship>()
                .map_err(|error| wasm_error!(WasmErrorInner::Guest(error.to_string())))?;

            let Some(relationship) = relationship else {
                return Ok(ValidateCallbackResult::Invalid(
                    "Trust subject link target must contain a trust relationship".into(),
                ));
            };

            if relationship.subject_agent != subject {
                return Ok(ValidateCallbackResult::Invalid(
                    "Trust subject link base must match the relationship subject".into(),
                ));
            }

            if record.action().author() != action.author() {
                return Ok(ValidateCallbackResult::Invalid(
                    "Trust subject link author must match the relationship author".into(),
                ));
            }

            Ok(ValidateCallbackResult::Valid)
        }

        FlatOp::Link(OpLink::CreateLink {
            link_type: LinkTypes::RelationshipToRemoval,
            action,
        }) => {
            let Some(relationship_hash) = action.base_address.clone().into_action_hash() else {
                return Ok(ValidateCallbackResult::Invalid(
                    "Trust removal link base must be a relationship action hash".into(),
                ));
            };

            let Some(removal_hash) = action.target_address.clone().into_action_hash() else {
                return Ok(ValidateCallbackResult::Invalid(
                    "Trust removal link target must be a removal action hash".into(),
                ));
            };

            let original = must_get_valid_record(relationship_hash.clone())?;
            let relationship = original
                .entry()
                .to_app_option::<TrustRelationship>()
                .map_err(|error| wasm_error!(WasmErrorInner::Guest(error.to_string())))?;

            if relationship.is_none() {
                return Ok(ValidateCallbackResult::Invalid(
                    "Trust removal link base must reference a trust relationship".into(),
                ));
            }

            if original.action().author() != action.author() {
                return Ok(ValidateCallbackResult::Invalid(
                    "Only the original relationship author can create its removal link".into(),
                ));
            }

            let removal_record = must_get_valid_record(removal_hash)?;

            if removal_record.action().author() != action.author() {
                return Ok(ValidateCallbackResult::Invalid(
                    "Trust removal link author must match the removal author".into(),
                ));
            }

            let removal = removal_record
                .entry()
                .to_app_option::<TrustRemoval>()
                .map_err(|error| wasm_error!(WasmErrorInner::Guest(error.to_string())))?;

            let Some(removal) = removal else {
                return Ok(ValidateCallbackResult::Invalid(
                    "Trust removal link target must contain a removal record".into(),
                ));
            };

            if removal.relationship_action != relationship_hash {
                return Ok(ValidateCallbackResult::Invalid(
                    "Trust removal link must reference the relationship being removed".into(),
                ));
            }

            Ok(ValidateCallbackResult::Valid)
        }

        FlatOp::Link(OpLink::DeleteLink { .. }) => Ok(ValidateCallbackResult::Invalid(
            "Trust graph history links cannot be deleted".into(),
        )),

        FlatOp::Update(OpUpdate::Entry {
            app_entry: EntryTypes::TrustRelationship(_),
            ..
        }) => Ok(ValidateCallbackResult::Invalid(
            "Published trust relationships cannot be edited".into(),
        )),

        FlatOp::Update(OpUpdate::Entry {
            app_entry: EntryTypes::TrustRemoval(_),
            ..
        }) => Ok(ValidateCallbackResult::Invalid(
            "Trust removal records cannot be edited".into(),
        )),

        FlatOp::Delete(OpDelete { action }) => {
            let original = must_get_valid_record(action.deletes_address.clone())?;

            let relationship = original
                .entry()
                .to_app_option::<TrustRelationship>()
                .map_err(|error| wasm_error!(WasmErrorInner::Guest(error.to_string())))?;

            let removal = original
                .entry()
                .to_app_option::<TrustRemoval>()
                .map_err(|error| wasm_error!(WasmErrorInner::Guest(error.to_string())))?;

            if relationship.is_some() || removal.is_some() {
                return Ok(ValidateCallbackResult::Invalid(
                    "Published trust graph records cannot be deleted".into(),
                ));
            }

            Ok(ValidateCallbackResult::Valid)
        }

        _ => Ok(ValidateCallbackResult::Valid),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        validate_trust_relationship, validate_trust_removal, TrustContext, TrustRelationship,
        TrustRemoval,
    };
    use hdi::prelude::*;

    fn test_agent(byte: u8) -> AgentPubKey {
        AgentPubKey::from_raw_32(vec![byte; 32])
    }

    #[test]
    fn accepts_supported_trust_contexts() {
        let author = test_agent(1);
        let subject = test_agent(2);

        let contexts = vec![
            TrustContext::General,
            TrustContext::Commerce,
            TrustContext::Professional,
            TrustContext::Hospitality,
            TrustContext::Technical,
            TrustContext::Community,
            TrustContext::Reliability,
        ];

        for context in contexts {
            let relationship = TrustRelationship {
                schema_version: 1,
                subject_agent: subject.clone(),
                context,
            };

            assert!(validate_trust_relationship(&relationship, &author).is_ok());
        }
    }

    #[test]
    fn rejects_unsupported_relationship_schema() {
        let relationship = TrustRelationship {
            schema_version: 2,
            subject_agent: test_agent(2),
            context: TrustContext::General,
        };

        assert!(validate_trust_relationship(&relationship, &test_agent(1)).is_err());
    }

    #[test]
    fn rejects_self_trust_relationship() {
        let author = test_agent(1);

        let relationship = TrustRelationship {
            schema_version: 1,
            subject_agent: author.clone(),
            context: TrustContext::General,
        };

        assert!(validate_trust_relationship(&relationship, &author).is_err());
    }

    #[test]
    fn accepts_valid_removal() {
        let removal = TrustRemoval {
            schema_version: 1,
            relationship_action: ActionHash::from_raw_32(vec![3; 32]),
        };

        assert!(validate_trust_removal(&removal).is_ok());
    }

    #[test]
    fn rejects_unsupported_removal_schema() {
        let removal = TrustRemoval {
            schema_version: 2,
            relationship_action: ActionHash::from_raw_32(vec![3; 32]),
        };

        assert!(validate_trust_removal(&removal).is_err());
    }
}
