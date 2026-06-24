#![allow(dead_code)]

#[path = "../src/hardware/mod.rs"]
mod hardware;

use hardware::cpu_boost::{apply_cpu_boost_mode_at, CpuBoostMode};
use std::fs;
use std::path::{Path, PathBuf};

fn temp_cpufreq_dir(name: &str) -> PathBuf {
    let dir =
        std::env::temp_dir().join(format!("asus-control-daemon-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("temp dir should be created");
    dir
}

fn write_policy(root: &Path, policy: &str, governors: &str, epp_values: &str) -> PathBuf {
    let policy_dir = root.join(policy);
    fs::create_dir_all(&policy_dir).expect("policy dir should be created");
    fs::write(policy_dir.join("scaling_available_governors"), governors).unwrap();
    fs::write(policy_dir.join("scaling_governor"), "schedutil").unwrap();
    fs::write(
        policy_dir.join("energy_performance_available_preferences"),
        epp_values,
    )
    .unwrap();
    fs::write(
        policy_dir.join("energy_performance_preference"),
        "balance_performance",
    )
    .unwrap();
    policy_dir
}

#[test]
fn cpu_boost_disabled_writes_global_boost_off() {
    let root = temp_cpufreq_dir("cpu-boost-disabled");
    fs::write(root.join("boost"), "1").unwrap();
    write_policy(
        &root,
        "policy0",
        "powersave schedutil performance",
        "power balance_power balance_performance performance",
    );

    apply_cpu_boost_mode_at(&root, CpuBoostMode::Disabled).expect("disabled should apply");

    assert_eq!(fs::read_to_string(root.join("boost")).unwrap(), "0");

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn cpu_boost_aggressive_writes_boost_and_performance_epp_without_changing_governor() {
    let root = temp_cpufreq_dir("cpu-boost-aggressive");
    fs::write(root.join("boost"), "0").unwrap();
    let policy = write_policy(
        &root,
        "policy0",
        "powersave schedutil performance",
        "power balance_power balance_performance performance",
    );

    apply_cpu_boost_mode_at(&root, CpuBoostMode::Aggressive).expect("aggressive should apply");

    assert_eq!(fs::read_to_string(root.join("boost")).unwrap(), "1");
    assert_eq!(
        fs::read_to_string(policy.join("scaling_governor")).unwrap(),
        "schedutil"
    );
    assert_eq!(
        fs::read_to_string(policy.join("energy_performance_preference")).unwrap(),
        "performance"
    );

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn cpu_boost_efficient_rejects_unavailable_epp_without_partial_boost_write() {
    let root = temp_cpufreq_dir("cpu-boost-efficient");
    fs::write(root.join("boost"), "0").unwrap();
    let policy = write_policy(
        &root,
        "policy0",
        "powersave schedutil performance",
        "balance_power balance_performance performance",
    );

    let error = apply_cpu_boost_mode_at(&root, CpuBoostMode::Efficient)
        .expect_err("efficient should be unavailable without exact power EPP");

    assert!(error.to_string().contains("unavailable CPU boost mode"));
    assert_eq!(fs::read_to_string(root.join("boost")).unwrap(), "0");
    assert_eq!(
        fs::read_to_string(policy.join("energy_performance_preference")).unwrap(),
        "balance_performance"
    );

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn cpu_boost_dynamic_epp_lock_rejects_epp_dependent_mode_without_partial_boost_write() {
    let root = temp_cpufreq_dir("cpu-boost-dynamic-epp");
    fs::write(root.join("boost"), "0").unwrap();
    let policy = write_policy(
        &root,
        "policy0",
        "powersave schedutil performance",
        "power balance_power balance_performance performance",
    );
    fs::write(policy.join("dynamic_epp"), "1").unwrap();

    let error = apply_cpu_boost_mode_at(&root, CpuBoostMode::Aggressive)
        .expect_err("dynamic epp lock should reject epp-dependent mode");

    assert!(error.to_string().contains("unavailable CPU boost mode"));
    assert_eq!(fs::read_to_string(root.join("boost")).unwrap(), "0");
    assert_eq!(
        fs::read_to_string(policy.join("energy_performance_preference")).unwrap(),
        "balance_performance"
    );

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn cpu_boost_enabled_restores_first_observed_system_epp_baseline() {
    let root = temp_cpufreq_dir("cpu-boost-enabled-baseline");
    fs::write(root.join("boost"), "0").unwrap();
    let policy = write_policy(
        &root,
        "policy0",
        "powersave schedutil performance",
        "power balance_power balance_performance performance",
    );
    fs::write(
        policy.join("energy_performance_preference"),
        "balance_power",
    )
    .unwrap();

    apply_cpu_boost_mode_at(&root, CpuBoostMode::Aggressive).expect("aggressive should apply");
    apply_cpu_boost_mode_at(&root, CpuBoostMode::Enabled).expect("enabled should apply");

    assert_eq!(fs::read_to_string(root.join("boost")).unwrap(), "1");
    assert_eq!(
        fs::read_to_string(policy.join("scaling_governor")).unwrap(),
        "schedutil"
    );
    assert_eq!(
        fs::read_to_string(policy.join("energy_performance_preference")).unwrap(),
        "balance_power"
    );

    let _ = fs::remove_dir_all(&root);
}
