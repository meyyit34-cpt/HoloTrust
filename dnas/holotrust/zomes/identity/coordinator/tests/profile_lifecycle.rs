
use holochain::prelude::{ActionHash, Record};
use holochain::sweettest::*;
use identity_coordinator::{CreateProfileInput, UpdateProfileInput};
use identity_integrity::AgentType;
use std::path::PathBuf;

async fn setup_single_agent() -> (SweetConductorBatch, Vec<SweetApp>) {
    let dna_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../workdir/holotrust.dna");

    assert!(
        dna_path.exists(),
        "Packaged DNA not found at: {}",
        dna_path.display()
    );

    let dna = SweetDnaFile::from_bundle(&dna_path)
        .await
        .expect("Failed to load DNA bundle");

    let mut conductors = SweetConductorBatch::standard(1).await;

    let apps = conductors
        .setup_app("holotrust", [&dna])
        .await
        .expect("Failed to install app");

    (conductors, apps)
}

#[tokio::test(flavor = "multi_thread")]
async fn invalid_profile_updates_do_not_replace_latest_profile() {
    let (mut conductors, apps) = setup_single_agent().await;
    let zome = apps[0].cells()[0].zome("identity_coordinator");

    let original_hash: ActionHash = conductors[0]
        .sweet_handle()
        .call(
            &zome,
            "create_profile",
            CreateProfileInput {
                display_name: "Mehmet".into(),
                agent_type: AgentType::Human,
                description: Some("Valid profile".into()),
            },
        )
        .await;

    let invalid_updates = vec![
        ("".to_string(), None),
        ("a".repeat(65), None),
        ("Mehmet\nAdmin".to_string(), None),
        ("Mehmet".to_string(), Some("a".repeat(281))),
        (
            "Mehmet".to_string(),
            Some("Invalid\nDescription".to_string()),
        ),
    ];

    for (display_name, description) in invalid_updates {
        let result: Result<ActionHash, _> = conductors[0]
            .sweet_handle()
            .call_fallible(
                &zome,
                "update_profile",
                UpdateProfileInput {
                    action_hash: original_hash.clone(),
                    display_name,
                    agent_type: AgentType::Human,
                    description,
                },
            )
            .await;

        assert!(result.is_err(), "Invalid profile update was accepted");

        let current_profile: Option<Record> = conductors[0]
            .sweet_handle()
            .call(&zome, "get_my_profile", ())
            .await;

        let current_profile = current_profile.expect("Existing profile disappeared");

        assert_eq!(
            current_profile.action_address(),
            &original_hash,
            "Invalid update changed the current profile"
        );
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn duplicate_profile_creation_is_rejected_and_original_remains_available() {
    let (mut conductors, apps) = setup_single_agent().await;
    let zome = apps[0].cells()[0].zome("identity_coordinator");

    let original_hash: ActionHash = conductors[0]
        .sweet_handle()
        .call(
            &zome,
            "create_profile",
            CreateProfileInput {
                display_name: "Original".into(),
                agent_type: AgentType::Human,
                description: None,
            },
        )
        .await;

    let duplicate: Result<ActionHash, _> = conductors[0]
        .sweet_handle()
        .call_fallible(
            &zome,
            "create_profile",
            CreateProfileInput {
                display_name: "Duplicate".into(),
                agent_type: AgentType::Human,
                description: None,
            },
        )
        .await;

    assert!(
        duplicate.is_err(),
        "An agent must not create a second profile"
    );

    let current_profile: Option<Record> = conductors[0]
        .sweet_handle()
        .call(&zome, "get_my_profile", ())
        .await;

    let current_profile = current_profile.expect("Original profile disappeared");

    assert_eq!(
        current_profile.action_address(),
        &original_hash,
        "Duplicate creation replaced the original profile"
    );
}

// END OF FILE
