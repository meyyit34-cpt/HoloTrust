/// Checks the legacy review author's previous source-chain activity.
/// Legacy reviews remain separate from contextual reviews.
fn validate_review_history(
    author: &AgentPubKey,
    previous_action: Option<ActionHash>,
    subject_agent: &AgentPubKey,
) -> ExternResult<ValidateCallbackResult> {
    let Some(chain_top) = previous_action else {
        return Ok(ValidateCallbackResult::Invalid(
            "A reputation review must follow an existing source-chain action".into(),
        ));
    };

    let review_entry_type = EntryType::App(UnitEntryTypes::ReputationReview.try_into()?);
    let previous_activity = must_get_agent_activity(author.clone(), ChainFilter::new(chain_top))?;

    for activity in previous_activity {
        if activity.action.hashed.content.entry_type() != Some(&review_entry_type) {
            continue;
        }

        let previous_record = must_get_valid_record(activity.action.hashed.hash.clone())?;

        let previous_review = previous_record
            .entry()
            .to_app_option::<ReputationReview>()
            .map_err(|e| wasm_error!(WasmErrorInner::Guest(e.to_string())))?;

        let Some(previous_review) = previous_review else {
            continue;
        };

        if previous_review.subject_agent == *subject_agent {
            return Ok(ValidateCallbackResult::Invalid(
                "An agent can publish only one reputation review per subject".into(),
            ));
        }
    }

    Ok(ValidateCallbackResult::Valid)
}
