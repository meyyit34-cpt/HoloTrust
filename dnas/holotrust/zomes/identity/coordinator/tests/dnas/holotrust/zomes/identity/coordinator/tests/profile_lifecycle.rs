use holochain::prelude::ActionHash;
use holochain::sweettest::*;
use identity_coordinator::{CreateProfileInput, UpdateProfileInput};
use identity_integrity::AgentType;
use std::path::PathBuf;

#[tokio::test(flavor = "multi_thread")]
async fn invalid_profile_updates_do_not_replace_latest_profile() {
    let dna_path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../workdir/holotrust.dna");

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

        let current_profile = conductors[0]
            .sweet_handle()
            .call(
                &zome,
                "get_my_profile",
                (),
            )
            .await;

        let current_profile = current_profile.expect("Existing profile disappeared");

        assert_eq!(
            current_profile.action_address(),
            &original_hash,
            "Invalid update changed the current profile"
        );
    }
}

// END OF FILE
