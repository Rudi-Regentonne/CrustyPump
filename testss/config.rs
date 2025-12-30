// tests/config_integration.rs
use heatpump::structs::Config;
use std::env;

#[test]
fn config_from_env_integration() {
    env::set_var("HEATPUMP_IP", "10.0.0.1");
    env::set_var("PIN", "0000");

    let cfg = Config::from_env().expect("config read");
    assert_eq!(cfg.device_ip, "10.0.0.1");
    assert_eq!(cfg.pin, "0000");

    env::remove_var("HEATPUMP_IP");
    env::remove_var("PIN");
}
