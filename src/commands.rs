use crate::{
    client::KeygenClient,
    err::ErrorSummary,
    licensed::{License, LicensedState},
    machine::Machine,
};
use tauri::{command, AppHandle, Runtime, State, Window};
use tokio::sync::Mutex;

type Result<T> = std::result::Result<T, ErrorSummary>;

#[command]
pub async fn has_valid_license<R: Runtime>(
    _app: AppHandle<R>,
    _window: Window<R>,
    licensed_state: State<'_, Mutex<LicensedState>>,
) -> Result<bool> {
    let licensed_state = licensed_state.lock().await;

    Ok(licensed_state.has_valid_license())
}

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
pub async fn can_update<R: Runtime>(
    _app: AppHandle<R>,
    _window: Window<R>,
    licensed_state: State<'_, Mutex<LicensedState>>,
) -> Result<bool> {
    let licensed_state = licensed_state.lock().await;

    match licensed_state.get_license() {
        Some(license) => match license.has_not_expired() {
            Ok(not_expired) => Ok(not_expired),
            Err(err) => {
                dbg!(&err);
                Ok(false)
            }
        },
        None => Ok(false),
    }
}

#[command]
pub async fn validate<R: Runtime>(
    app: AppHandle<R>,
    _window: Window<R>,
    machine: State<'_, Mutex<Machine>>,
    client: State<'_, Mutex<KeygenClient>>,
    licensed_state: State<'_, Mutex<LicensedState>>,
    key: String,
    cache_response: bool,
) -> Result<License> {
    let machine = machine.lock().await;
    let client = client.lock().await;

    let mut licensed_state = licensed_state.lock().await;

    match licensed_state.validate(key, &machine, &client).await {
        Ok((license, res_cache)) => {
            // update app state
            licensed_state.update(license.clone(), &app)?;

            // cache response
            if license.valid && cache_response && license.expiry.is_some() {
                LicensedState::cache_response(&app, license.key.clone(), res_cache)?;
            }

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
    app: AppHandle<R>,
    _window: Window<R>,
    machine: State<'_, Mutex<Machine>>,
    client: State<'_, Mutex<KeygenClient>>,
    licensed_state: State<'_, Mutex<LicensedState>>,
) -> Result<License> {
    let machine = machine.lock().await;
    let client = client.lock().await;

    let mut licensed_state = licensed_state.lock().await;

    match machine.activate(&mut licensed_state, &client).await {
        Ok(license) => {
            licensed_state.update(license.clone(), &app)?;

            Ok(license)
        }
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
    ttl: u32,
) -> Result<()> {
    let machine = machine.lock().await;
    let client = client.lock().await;

    let licensed_state = licensed_state.lock().await;

    match machine.checkout(&licensed_state, &client, &app, ttl).await {
        Ok(()) => Ok(()),
        Err(err) => {
            dbg!(&err);
            Err(err.into())
        }
    }
}
