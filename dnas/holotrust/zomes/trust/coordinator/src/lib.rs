use hdk::prelude::*;
use trust_integrity::{
    EntryTypes, LinkTypes, TrustContext, TrustRelationship, TrustRemoval,
};

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateTrustRelationshipInput {
    pub subject_agent: AgentPubKey,
    pub context: TrustContext,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RemoveTrustRelationshipInput {
    pub relationship_action: ActionHash,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetTrustRelationshipInput {
    pub action_hash: ActionHash,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetTrustRelationshipsByAuthorInput {
    pub author: AgentPubKey,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetTrustRelationshipsForSubjectInput {
    pub subject_agent: AgentPubKey,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetTrustRelationshipsByContextInput {
    pub author: AgentPubKey,
    pub context: TrustContext,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetTrustRelationshipHistoryInput {
    pub relationship_action: ActionHash,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TrustRelationshipHistory {
    pub relationship: Record,
    pub removal: Option<Record>,
}

fn decode_relationship(record: &Record) -> ExternResult<Option<TrustRelationship>> {
    record
        .entry()
        .to_app_option::<TrustRelationship>()
        .map_err(|error| wasm_error!(WasmErrorInner::Guest(error.to_string())))
}

fn decode_removal(record: &Record) -> ExternResult<Option<TrustRemoval>> {
    record
        .entry()
        .to_app_option::<TrustRemoval>()
        .map_err(|error| wasm_error!(WasmErrorInner::Guest(error.to_string())))
}

fn get_removal_record(
    relationship_action: &ActionHash,
    author: &AgentPubKey,
) -> ExternResult<Option<Record>> {
    let links = get_links(
        LinkQuery::try_new(
            relationship_action.clone(),
            LinkTypes::RelationshipToRemoval,
        )?,
        GetStrategy::default(),
    )?;

    for link in links {
        if &link.author != author {
            continue;
        }

        let Some(removal_hash) = link.target.into_action_hash() else {
            continue;
        };

        let Some(record) = get(removal_hash, GetOptions::default())? else {
            continue;
        };

        if record.action().author() != author {
            continue;
        }

        let Some(removal) = decode_removal(&record)? else {
            continue;
        };

        if &removal.relationship_action == relationship_action {
            return Ok(Some(record));
        }
    }

    Ok(None)
}

fn relationship_history(
    relationship: Record,
) -> ExternResult<TrustRelationshipHistory> {
    let author = relationship.action().author().clone();
    let relationship_action = relationship.action_address().clone();

    let removal = get_removal_record(&relationship_action, &author)?;

    Ok(TrustRelationshipHistory {
        relationship,
        removal,
    })
}

#[hdk_extern]
pub fn create_trust_relationship(
    input: CreateTrustRelationshipInput,
) -> ExternResult<ActionHash> {
    let author = agent_info()?.agent_initial_pubkey;

    let relationship = TrustRelationship {
        schema_version: 1,
        subject_agent: input.subject_agent.clone(),
        context: input.context,
    };

    trust_integrity::validate_trust_relationship(&relationship, &author)
        .map_err(|reason| wasm_error!(WasmErrorInner::Guest(reason)))?;

    let relationship_action =
        create_entry(EntryTypes::TrustRelationship(relationship))?;

    create_link(
        author,
        relationship_action.clone(),
        LinkTypes::AuthorToTrustRelationship,
        "trust",
    )?;

    create_link(
        input.subject_agent,
        relationship_action.clone(),
        LinkTypes::SubjectToTrustRelationship,
        "trust",
    )?;

    Ok(relationship_action)
}

#[hdk_extern]
pub fn remove_trust_relationship(
    input: RemoveTrustRelationshipInput,
) -> ExternResult<ActionHash> {
    let author = agent_info()?.agent_initial_pubkey;

    let Some(original) = get(
        input.relationship_action.clone(),
        GetOptions::default(),
    )? else {
        return Err(wasm_error!(WasmErrorInner::Guest(
            "Trust relationship not found".into(),
        )));
    };

    if decode_relationship(&original)?.is_none() {
        return Err(wasm_error!(WasmErrorInner::Guest(
            "Target record is not a trust relationship".into(),
        )));
    }

    if original.action().author() != &author {
        return Err(wasm_error!(WasmErrorInner::Guest(
            "Only the original author can remove this trust relationship".into(),
        )));
    }

    if get_removal_record(&input.relationship_action, &author)?.is_some() {
        return Err(wasm_error!(WasmErrorInner::Guest(
            "Trust relationship has already been removed".into(),
        )));
    }

    let removal = TrustRemoval {
        schema_version: 1,
        relationship_action: input.relationship_action.clone(),
    };

    trust_integrity::validate_trust_removal(&removal)
        .map_err(|reason| wasm_error!(WasmErrorInner::Guest(reason)))?;

    let removal_action = create_entry(EntryTypes::TrustRemoval(removal))?;

    create_link(
        input.relationship_action,
        removal_action.clone(),
        LinkTypes::RelationshipToRemoval,
        "removal",
    )?;

    Ok(removal_action)
}

#[hdk_extern]
pub fn get_trust_relationship(
    input: GetTrustRelationshipInput,
) -> ExternResult<Option<Record>> {
    let Some(record) = get(input.action_hash, GetOptions::default())? else {
        return Ok(None);
    };

    if decode_relationship(&record)?.is_none() {
        return Ok(None);
    }

    Ok(Some(record))
}

#[hdk_extern]
pub fn get_trust_relationships_by_author(
    input: GetTrustRelationshipsByAuthorInput,
) -> ExternResult<Vec<Record>> {
    let links = get_links(
        LinkQuery::try_new(
            input.author.clone(),
            LinkTypes::AuthorToTrustRelationship,
        )?,
        GetStrategy::default(),
    )?;

    let mut records = Vec::new();

    for link in links {
        if link.author != input.author {
            continue;
        }

        let Some(action_hash) = link.target.into_action_hash() else {
            continue;
        };

        let Some(record) = get(action_hash, GetOptions::default())? else {
            continue;
        };

        if record.action().author() != &input.author {
            continue;
        }

        let Some(relationship) = decode_relationship(&record)? else {
            continue;
        };

        if relationship.subject_agent == input.author {
            continue;
        }

        records.push(record);
    }

    records.sort_by_key(|record| record.action().timestamp());

    Ok(records)
}

#[hdk_extern]
pub fn get_trust_relationships_for_subject(
    input: GetTrustRelationshipsForSubjectInput,
) -> ExternResult<Vec<Record>> {
    let links = get_links(
        LinkQuery::try_new(
            input.subject_agent.clone(),
            LinkTypes::SubjectToTrustRelationship,
        )?,
        GetStrategy::default(),
    )?;

    let mut records = Vec::new();

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

        let Some(relationship) = decode_relationship(&record)? else {
            continue;
        };

        if relationship.subject_agent != input.subject_agent {
            continue;
        }

        records.push(record);
    }

    records.sort_by_key(|record| record.action().timestamp());

    Ok(records)
}

#[hdk_extern]
pub fn get_trust_relationships_by_context(
    input: GetTrustRelationshipsByContextInput,
) -> ExternResult<Vec<Record>> {
    let records = get_trust_relationships_by_author(
        GetTrustRelationshipsByAuthorInput {
            author: input.author,
        },
    )?;

    let mut filtered = Vec::new();

    for record in records {
        let Some(relationship) = decode_relationship(&record)? else {
            continue;
        };

        if relationship.context == input.context {
            filtered.push(record);
        }
    }

    Ok(filtered)
}

#[hdk_extern]
pub fn get_trust_relationship_history(
    input: GetTrustRelationshipHistoryInput,
) -> ExternResult<Option<TrustRelationshipHistory>> {
    let Some(record) = get(
        input.relationship_action,
        GetOptions::default(),
    )? else {
        return Ok(None);
    };

    if decode_relationship(&record)?.is_none() {
        return Ok(None);
    }

    Ok(Some(relationship_history(record)?))
}
