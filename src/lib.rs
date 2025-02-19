use napi::{bindgen_prelude::*, Error as NapiError, Result};
use napi_derive::napi;
use winmix::WinMix;
use serde::Serialize;

// Helper function to convert errors into NapiError
fn convert_error<E: std::fmt::Display>(err: E) -> NapiError {
    NapiError::from_reason(err.to_string())
}

#[derive(Serialize)]
struct AudioSession {
    pid: u32,
    path: String,
    volume: f32,
    muted: bool,
}

#[napi]
pub fn list_audio_sessions() -> Result<String> {
    unsafe {
        let winmix = WinMix::default();
        let mut sessions = Vec::new();

        for session in winmix.enumerate().map_err(convert_error)? {
            let volume = session.vol.get_master_volume().map_err(convert_error)?;
            let muted = session.vol.get_mute().map_err(convert_error)?;

            sessions.push(AudioSession {
                pid: session.pid,
                path: session.path,
                volume,
                muted,
            });
        }

        // Serialize the vector of sessions into JSON
        serde_json::to_string(&sessions).map_err(|e| NapiError::from_reason(e.to_string()))
    }
}

#[napi]
pub fn set_master_volume(volume: u8) -> Result<String> {
    unsafe {
        let winmix = WinMix::default();
        let normalized = volume as f32 / 100.0;
        if let Some(master_session) = winmix.enumerate().map_err(convert_error)?.into_iter().next() {
            master_session.vol.set_master_volume(normalized).map_err(convert_error)?;
            Ok(format!("Master volume set to {}% (normalized: {:.2})", volume, normalized))
        } else {
            Ok("No master session found".to_string())
        }
    }
}

#[napi]
pub fn mute_master_volume(mute: bool) -> Result<String> {
    unsafe {
        let winmix = WinMix::default();
        if let Some(master_session) = winmix.enumerate().map_err(convert_error)?.into_iter().next() {
            master_session.vol.set_mute(mute).map_err(convert_error)?;
            Ok(format!("Master volume muted: {}", mute))
        } else {
            Ok("No master session found".to_string())
        }
    }
}

#[napi]
pub fn set_app_volume(app_name: String, volume: u8) -> Result<String> {
    unsafe {
        let winmix = WinMix::default();
        let normalized = volume as f32 / 100.0;
        if let Some(session) = winmix
            .enumerate()
            .map_err(convert_error)?
            .into_iter()
            .find(|s| s.path.to_lowercase().contains(&app_name.to_lowercase()))
        {
            session.vol.set_master_volume(normalized).map_err(convert_error)?;
            Ok(format!(
                "Volume for '{}' set to {}% (normalized: {:.2})",
                app_name, volume, normalized
            ))
        } else {
            Ok(format!("Could not find an application named '{}'", app_name))
        }
    }
}