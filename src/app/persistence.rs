use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use crate::app::state::{SessionData, SessionsModel};
use crate::constants::SESSION_COUNT_MAX;
use crate::model::error::{SwarmError, SwarmResult};
use crate::model::{identifier, node, storage};

const ACTIVE_FILE_NAME: &str = "active";
const SESSION_LAST_FILE_NAME: &str = "last_session.json";
const SESSION_EXTENSION: &str = "json";
const SESSIONS_DIRECTORY_NAME: &str = "sessions";

pub struct SessionsReport {
    pub sessions: SessionsModel,
    pub skipped_count: u32,
}

fn active_identifier(directory: &Path) -> Option<String> {
    let text = fs::read_to_string(directory.join(ACTIVE_FILE_NAME)).ok()?;
    let identifier = text.trim();

    if !identifier::is_valid(identifier) {
        return None;
    }

    Some(identifier.to_owned())
}

pub fn delete_session_file(identifier: &str) -> SwarmResult<()> {
    if !identifier::is_valid(identifier) {
        return Err(SwarmError::Validation(format!(
            "the session identifier '{identifier}' is not a valid file name",
        )));
    }

    let path = session_path(&sessions_directory()?, identifier);

    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.into()),
    }
}

pub fn delete_sessions_directory() -> SwarmResult<()> {
    let directory = sessions_directory()?;

    match fs::remove_dir_all(&directory) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.into()),
    }
}

fn load_last_session() -> Option<SessionData> {
    let path = storage::application_directory()
        .ok()?
        .join(SESSION_LAST_FILE_NAME);

    let content = fs::read_to_string(&path).ok()?;

    let outcome = serde_json::from_str::<SessionData>(&content)
        .map_err(SwarmError::from)
        .and_then(validated);

    match outcome {
        Ok(session) => Some(session),
        Err(error) => {
            eprintln!("Ignoring the last session in {}: {error}", path.display());

            None
        }
    }
}

pub fn load_sessions() -> SessionsReport {
    let mut sessions = SessionsModel::default();
    let mut skipped_count: u32 = 0;

    if let Ok(directory) = sessions_directory() {
        skipped_count = read_sessions(&directory, &mut sessions);
        sessions.rebuild_order();

        if !sessions.is_empty() {
            select_active_session(&directory, &mut sessions);
        }
    }

    if let Some(last) = load_last_session() {
        sessions.set_last_closed(last);
    }

    SessionsReport {
        sessions,
        skipped_count,
    }
}

fn read_session_file(path: &Path) -> SwarmResult<SessionData> {
    let content = fs::read_to_string(path)?;
    let session = validated(serde_json::from_str(&content)?)?;
    let stem = path.file_stem().and_then(|stem| stem.to_str());

    if stem != Some(session.identifier.as_str()) {
        return Err(SwarmError::Validation(
            "the session identifier does not match the file name".to_owned(),
        ));
    }

    Ok(session)
}

fn read_sessions(directory: &Path, sessions: &mut SessionsModel) -> u32 {
    let Ok(entries) = fs::read_dir(directory) else {
        return 0;
    };

    let mut examined_count: u32 = 0;
    let mut skipped_count: u32 = 0;

    for entry in entries.flatten() {
        let path = entry.path();

        if path.extension().and_then(|extension| extension.to_str()) != Some(SESSION_EXTENSION) {
            continue;
        }

        examined_count += 1;

        if examined_count > SESSION_COUNT_MAX {
            skipped_count += 1;

            continue;
        }

        match read_session_file(&path) {
            Ok(session) => sessions.add_session(session),
            Err(error) => {
                eprintln!("Skipped the session {}: {error}", path.display());

                skipped_count += 1;
            }
        }
    }

    debug_assert!(skipped_count <= examined_count);

    skipped_count
}

pub fn save_last_session(session: &SessionData) -> SwarmResult<()> {
    let path = storage::application_directory()?.join(SESSION_LAST_FILE_NAME);
    let json = serde_json::to_string_pretty(session)?;

    assert_ne!(json, "");

    storage::write_atomic(&path, json.as_bytes())
}

pub fn save_sessions(sessions: &SessionsModel) -> SwarmResult<()> {
    let directory = sessions_directory()?;

    for session in sessions.ordered() {
        let json = serde_json::to_string_pretty(session)?;

        assert_ne!(json, "");

        storage::write_atomic(
            &session_path(&directory, &session.identifier),
            json.as_bytes(),
        )?;
    }

    if let Some(active) = sessions.active_identifier() {
        storage::write_atomic(&directory.join(ACTIVE_FILE_NAME), active.as_bytes())?;
    }

    Ok(())
}

fn select_active_session(directory: &Path, sessions: &mut SessionsModel) {
    let saved =
        active_identifier(directory).filter(|identifier| sessions.get(identifier).is_some());

    let active = saved.or_else(|| {
        sessions
            .ordered()
            .max_by_key(|session| session.last_modified)
            .map(|session| session.identifier.clone())
    });

    debug_assert!(active.is_some());

    sessions.set_active(active);
}

fn session_path(directory: &Path, identifier: &str) -> PathBuf {
    assert!(identifier::is_valid(identifier));

    directory.join(format!("{identifier}.{SESSION_EXTENSION}"))
}

fn sessions_directory() -> SwarmResult<PathBuf> {
    Ok(storage::application_directory()?.join(SESSIONS_DIRECTORY_NAME))
}

fn validated(mut session: SessionData) -> SwarmResult<SessionData> {
    if !identifier::is_valid(&session.identifier) {
        return Err(SwarmError::Validation(format!(
            "the session identifier '{}' is not a valid file name",
            session.identifier,
        )));
    }

    if !node::depth_is_bounded(&session.tree_state.nodes) {
        return Err(SwarmError::Validation(
            "the session tree is nested deeper than the supported limit".to_owned(),
        ));
    }

    node::visit_mut(&mut session.tree_state.nodes, |current, _| {
        current.recompute_name_cache();
    });

    session.search_state.ensure_parsed();
    session.tree_state.update_files_count();

    Ok(session)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_session_with_an_escaping_identifier_is_rejected() {
        let mut session = SessionData::new("escape".to_owned());

        session.identifier = "../../escape".to_owned();

        assert!(validated(session).is_err());
    }

    #[test]
    fn a_valid_session_passes_validation() {
        let session = SessionData::new("valid".to_owned());

        assert!(validated(session).is_ok());
    }

    #[test]
    fn deleting_an_escaping_identifier_is_refused() {
        assert!(delete_session_file("../options").is_err());
    }
}
