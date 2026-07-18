use std::path::Path;

use lupa::Language;
use magika::{ContentType, Session};

#[derive(Debug, Default)]
pub(crate) struct LanguageDetector {
    session: Option<Session>,
}

impl LanguageDetector {
    pub(crate) fn detect_content(&mut self, content: &[u8]) -> magika::Result<Option<Language>> {
        let file_type = self.session()?.identify_content_sync(content)?;
        Ok(language_from_content_type(file_type.content_type()))
    }

    pub(crate) fn detect_file(&mut self, path: &Path) -> magika::Result<Option<Language>> {
        let file_type = self.session()?.identify_file_sync(path)?;
        Ok(language_from_content_type(file_type.content_type()))
    }

    fn session(&mut self) -> magika::Result<&mut Session> {
        if self.session.is_none() {
            self.session = Some(Session::new()?);
        }
        Ok(self
            .session
            .as_mut()
            .expect("the Magika session was initialized above"))
    }
}

fn language_from_content_type(content_type: Option<ContentType>) -> Option<Language> {
    match content_type {
        Some(ContentType::Shell) => Some(Language::Bash),
        _ => None,
    }
}
