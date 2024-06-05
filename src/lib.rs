mod client;
mod commands;
mod err;
mod licensed;
mod machine;

use client::KeygenClient;
use err::Error;
use licensed::*;
use machine::Machine;
use std::fmt;
use tauri::{
    plugin::{Builder as PluginBuilder, TauriPlugin},
    Manager, Runtime,
};
use tokio::sync::Mutex;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Debug)]
pub struct KeygenVersion(pub u8, pub u8);

impl fmt::Display for KeygenVersion {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}.{}", self.0, self.1)
    }
}

#[derive(Clone)]
pub struct Builder {
    pub custom_domain: Option<String>,
    pub api_url: Option<String>,
    pub account_id: Option<String>,
    pub verify_key: String,
    pub version_header: Option<KeygenVersion>,
    pub cache_lifetime: i64, // in minutes
}

impl Builder {
    pub fn new(account_id: impl Into<String>, verify_key: impl Into<String>) -> Self {
        Self {
            custom_domain: None,
            api_url: Some("https://api.keygen.sh".into()),
            account_id: Some(account_id.into()),
            verify_key: verify_key.into(),
            version_header: None,
            cache_lifetime: 240,
        }
    }

    pub fn with_custom_domain(
        custom_domain: impl Into<String>,
        verify_key: impl Into<String>,
    ) -> Self {
        Self {
            custom_domain: Some(custom_domain.into()),
            account_id: None,
            api_url: None,
            verify_key: verify_key.into(),
            version_header: None,
            cache_lifetime: 240,
        }
    }

    pub fn api_url(mut self, api_url: impl Into<String>) -> Self {
        if self.custom_domain.is_none() {
            self.api_url = Some(api_url.into());
        }
        self
    }

    pub fn version_header(mut self, version_header: KeygenVersion) -> Self {
        self.version_header = Some(version_header);
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
                    self.custom_domain,
                    self.api_url,
                    self.account_id,
                    self.verify_key,
                    self.version_header,
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
