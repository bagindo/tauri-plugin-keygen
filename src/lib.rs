mod client;
mod commands;
mod err;
mod licensed;
mod machine;

use client::*;
use err::Error;
use licensed::*;
use machine::Machine;
use tauri::{
    plugin::{Builder as PluginBuilder, TauriPlugin},
    Manager, Runtime,
};
use tokio::sync::Mutex;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone)]
pub struct Builder {
    pub account_id: String,
    pub verify_key: String,
    pub api_url: String,
    pub api_version: String,
    pub cache_lifetime: i64, // in minutes
}

impl Builder {
    pub fn new(account_id: impl Into<String>, verify_key: impl Into<String>) -> Self {
        Self {
            account_id: account_id.into(),
            verify_key: verify_key.into(),
            api_url: "https://api.keygen.sh".into(),
            api_version: "v1".into(),
            cache_lifetime: 240,
        }
    }

    pub fn api_url(mut self, api_url: impl Into<String>) -> Self {
        self.api_url = api_url.into();
        self
    }

    pub fn api_version(mut self, api_version: impl Into<String>) -> Self {
        let v = api_version.into();
        self.api_version = if v.is_empty() { "v1".to_string() } else { v };
        self
    }

    pub fn cache_lifetime(mut self, cache_lifetime: i64) -> Self {
        self.cache_lifetime = if cache_lifetime < 60 {
            60 // min: 1 hour
        } else if cache_lifetime > 525_600 {
            525_600 // max: 1 year
        } else {
            cache_lifetime
        };

        self
    }

    pub fn build<R: Runtime>(self) -> TauriPlugin<R> {
        PluginBuilder::new("keygen")
            .invoke_handler(tauri::generate_handler![
                commands::get_license,
                commands::get_license_key,
                commands::validate_key,
                commands::activate,
                commands::checkout_machine,
            ])
            .setup(move |app| {
                // get app info
                let app_name = app.package_info().name.clone();
                let app_version = app.package_info().version.to_string();

                // init machine
                let machine = Machine::new(app_name, app_version);

                // init keygen client
                let keygen_client = KeygenClient::new(
                    self.account_id,
                    self.verify_key,
                    self.api_url,
                    self.api_version,
                    self.cache_lifetime,
                    machine.user_agent.clone(),
                );

                // init state
                match LicensedState::load(app, &keygen_client, &machine) {
                    Ok(licensed_state) => {
                        app.manage(Mutex::new(licensed_state));
                    }
                    Err(err) => {
                        dbg!(err);
                        app.manage(Mutex::new(LicensedState::default()));
                    }
                }
                app.manage(Mutex::new(machine));
                app.manage(Mutex::new(keygen_client));

                Ok(())
            })
            .build()
    }
}
