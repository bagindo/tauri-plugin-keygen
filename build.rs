const COMMANDS: &[&str] = &[
    "get_license",
    "get_license_key",
    "validate_key",
    "activate",
    "checkout_machine",
    "reset_license",
    "reset_license_key",
];

fn main() {
    tauri_plugin::Builder::new(COMMANDS).build();
}
