use identity_integrity::{validate_profile, AgentProfile, AgentType};

fn make_profile(name: &str, description: Option<&str>) -> AgentProfile {
    AgentProfile {
        schema_version: 1,
        display_name: name.to_string(),
        agent_type: AgentType::Human,
        description: description.map(str::to_string),
    }
}

#[test]
fn accepts_valid_profile() {
    assert!(validate_profile(&make_profile("Mehmet", Some("Valid profile"))).is_ok());
}

#[test]
fn rejects_unsupported_schema_version() {
    let mut profile = make_profile("Mehmet", None);
    profile.schema_version = 2;
    assert!(validate_profile(&profile).is_err());
}

#[test]
fn rejects_empty_display_name() {
    assert!(validate_profile(&make_profile("", None)).is_err());
}

#[test]
fn rejects_whitespace_only_display_name() {
    assert!(validate_profile(&make_profile("   ", None)).is_err());
}

#[test]
fn accepts_display_name_with_64_characters() {
    let name = "a".repeat(64);
    assert!(validate_profile(&make_profile(&name, None)).is_ok());
}

#[test]
fn rejects_display_name_with_65_characters() {
    let name = "a".repeat(65);
    assert!(validate_profile(&make_profile(&name, None)).is_err());
}

#[test]
fn rejects_control_characters_in_display_name() {
    assert!(validate_profile(&make_profile("Mehmet\nAdmin", None)).is_err());
}

#[test]
fn accepts_description_with_280_characters() {
    let description = "a".repeat(280);
    assert!(validate_profile(&make_profile("Mehmet", Some(&description))).is_ok());
}

#[test]
fn rejects_description_with_281_characters() {
    let description = "a".repeat(281);
    assert!(validate_profile(&make_profile("Mehmet", Some(&description))).is_err());
}

#[test]
fn rejects_control_characters_in_description() {
    assert!(validate_profile(&make_profile("Mehmet", Some("Valid\nInvalid"))).is_err());
}
