use crate::{
    client::KeygenClient,
    err::ErrorSummary,
    licensed::{license::License, LicensedState},
    machine::Machine,
};
use tauri::{command, AppHandle, Runtime, State, Window};
use tokio::sync::Mutex;

type Result<T> = std::result::Result<T, ErrorSummary>;

#[command]
pub async fn get_license<R: Runtime>(
    _app: AppHandle<R>,
    _window: Window<R>,
    licensed_state: State<'_, Mutex<LicensedState>>,
) -> Result<Option<License>> {
    let licensed_state = licensed_state.lock().await;
    Ok(licensed_state.get_license())
}

#[command]
pub async fn get_license_key<R: Runtime>(
    app: AppHandle<R>,
    _window: Window<R>,
) -> Result<Option<String>> {
    match LicensedState::get_cached_license_key(&app) {
        Ok(key) => Ok(key),
        Err(err) => {
            dbg!(&err);
            Err(err.into())
        }
    }
}

#[command]
#[allow(clippy::too_many_arguments)]
pub async fn validate_key<R: Runtime>(
    app: AppHandle<R>,
    _window: Window<R>,
    machine: State<'_, Mutex<Machine>>,
    client: State<'_, Mutex<KeygenClient>>,
    licensed_state: State<'_, Mutex<LicensedState>>,
    key: String,
    entitlements: Vec<String>,
    cache_valid_response: bool,
) -> Result<License> {
    let machine = machine.lock().await;
    let client = client.lock().await;

    let mut licensed_state = licensed_state.lock().await;

    match licensed_state
        .validate_key(key, entitlements, &machine, &client)
        .await
    {
        Ok((license, res_cache)) => {
            // cache valid response
            if license.valid && cache_valid_response && license.expiry.is_some() {
                LicensedState::cache_response(&app, &license.key, res_cache)?;
            }

            // update state
            licensed_state.update(Some(license.clone()));

            // cache license key
            LicensedState::cache_license_key(&license.key, &app)?;

            Ok(license)
        }
        Err(err) => {
            dbg!(&err);
            Err(err.into())
        }
    }
}

#[command]
pub async fn activate<R: Runtime>(
    _window: Window<R>,
    machine: State<'_, Mutex<Machine>>,
    client: State<'_, Mutex<KeygenClient>>,
    licensed_state: State<'_, Mutex<LicensedState>>,
) -> Result<()> {
    let machine = machine.lock().await;
    let client = client.lock().await;

    let mut licensed_state = licensed_state.lock().await;

    match machine.activate(&mut licensed_state, &client).await {
        Ok(()) => Ok(()),
        Err(err) => {
            dbg!(&err);
            Err(err.into())
        }
    }
}

#[command]
pub async fn checkout_machine<R: Runtime>(
    app: AppHandle<R>,
    _window: Window<R>,
    machine: State<'_, Mutex<Machine>>,
    licensed_state: State<'_, Mutex<LicensedState>>,
    client: State<'_, Mutex<KeygenClient>>,
    ttl_seconds: u32,
    ttl_forever: bool,
) -> Result<()> {
    let machine = machine.lock().await;
    let client = client.lock().await;

    let licensed_state = licensed_state.lock().await;

    match machine
        .checkout(&licensed_state, &client, &app, ttl_seconds, ttl_forever)
        .await
    {
        Ok(()) => Ok(()),
        Err(err) => {
            dbg!(&err);
            Err(err.into())
        }
    }
}

#[command]
pub async fn reset_license<R: Runtime>(
    app: AppHandle<R>,
    _window: Window<R>,
    licensed_state: State<'_, Mutex<LicensedState>>,
) -> Result<()> {
    let mut licensed_state = licensed_state.lock().await;

    // reset state
    licensed_state.update(None);

    // delete offline licenses
    Machine::remove_machine_file(&app)?;
    LicensedState::clear_response_cache(&app)?;

    Ok(())
}

#[command]
pub async fn reset_license_key<R: Runtime>(app: AppHandle<R>, _window: Window<R>) -> Result<()> {
    LicensedState::remove_cached_license_key(&app)?;
    Ok(())
}
