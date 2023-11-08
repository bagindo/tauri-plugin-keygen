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
    pub product_id: String,
    pub verify_key: String,
}

impl Builder {
    pub fn new(
        account_id: impl Into<String>,
        product_id: impl Into<String>,
        verify_key: impl Into<String>,
    ) -> Self {
        Self {
            account_id: account_id.into(),
            product_id: product_id.into(),
            verify_key: verify_key.into(),
        }
    }

    pub fn build<R: Runtime>(self) -> TauriPlugin<R> {
        PluginBuilder::new("keygen")
            .invoke_handler(tauri::generate_handler![
                commands::is_licensed,
                commands::get_license,
                commands::get_license_key,
                commands::validate,
                commands::activate,
                commands::checkout_machine,
                commands::can_update,
            ])
            .setup(|app| {
                // get app info
                let app_name = app.package_info().name.clone();
                let app_version = app.package_info().version.to_string();

                // init machine
                let machine = Machine::new(app_name, app_version);

                // init keygen client
                let keygen_client = KeygenClient::new(
                    self.account_id,
                    self.product_id,
                    self.verify_key,
                    machine.user_agent.clone(),
                );

                // init state
                match LicensedState::load(&app, &keygen_client, &machine) {
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
