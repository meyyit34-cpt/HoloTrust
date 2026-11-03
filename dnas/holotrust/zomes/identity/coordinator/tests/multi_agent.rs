use hdk::prelude::*;
use holochain::sweettest::{await_consistency, SweetConductorBatch, SweetDnaFile};
use identity_integrity::{AgentProfile, AgentType};
use std::path::PathBuf;

#[tokio::test(flavor = "multi_thread")]
async fn two_agents_share_profiles_and_cannot_modify_each_others_data() {
    let dna_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../workdir/holotrust.dna");

    assert!(
        dna_path.exists(),
        "Packaged DNA not found: {}",
        dna_path.display()
    );

    let dna_file = SweetDnaFile::from_bundle(&dna_path)
        .await
        .expect("Could not load HoloTrust DNA");

    let mut conductors = SweetConductorBatch::standard(2).await;

    let apps = conductors
        .setup_app("holotrust-test", &[dna_file])
        .await
        .expect("Could not install app");

    conductors.exchange_peer_info().await;

    let alice_cell = apps[0].cells()[0].clone();
    let bob_cell = apps[1].cells()[0].clone();

    let alice = alice_cell.zome("identity_coordinator");
    let bob = bob_cell.zome("identity_coordinator");

    let profile_hash: ActionHash = conductors[0]
        .call(
            &alice,
            "create_profile",
            identity_coordinator::CreateProfileInput {
                display_name: "Alice HoloTrust".into(),
                agent_type: AgentType::Human,
                description: Some("First network agent".into()),
            },
        )
        .await;

    assert!(!profile_hash.get_raw_36().is_empty());

    await_consistency(&[alice_cell.clone(), bob_cell.clone()])
        .await
        .expect("DHT consistency failed");

    let alice_profile: Option<Record> = conductors[1]
        .call(
            &bob,
            "get_profile",
            identity_coordinator::GetProfileInput {
                agent_pub_key: alice_cell.agent_pubkey().clone(),
            },
        )
        .await;

    let alice_profile = alice_profile.expect("Bob could not discover Alice's profile");

    let decoded_profile = alice_profile
        .entry()
        .to_app_option::<AgentProfile>()
        .expect("Could not decode profile")
        .expect("Profile entry missing");

    assert_eq!(decoded_profile.display_name, "Alice HoloTrust");

    let unauthorized_update = conductors[1]
        .call_fallible::<_, ActionHash>(
            &bob,
            "update_profile",
            identity_coordinator::UpdateProfileInput {
                action_hash: alice_profile.action_address().clone(),
                display_name: "Modified by Bob".into(),
                agent_type: AgentType::Human,
                description: Some("Unauthorized modification".into()),
            },
        )
        .await;

    assert!(
        unauthorized_update.is_err(),
        "SECURITY FAILURE: Bob modified Alice's profile"
    );

    let original_profile: Option<Record> = conductors[0].call(&alice, "get_my_profile", ()).await;

    let original_profile = original_profile.expect("Alice's profile disappeared");

    let original_decoded = original_profile
        .entry()
        .to_app_option::<AgentProfile>()
        .expect("Could not decode original profile")
        .expect("Original profile entry missing");

    assert_eq!(original_decoded.display_name, "Alice HoloTrust");

    await_consistency(&[alice_cell, bob_cell])
        .await
        .expect("DHT consistency failed after update attempt");
}
