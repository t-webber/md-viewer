use std::sync::Mutex;

use crate::state::unlock;

use super::{DriveFile, create_folder};

#[derive(Debug)]
enum AppFolder {
    Name(&'static str),
    Info(DriveFile),
}

impl AppFolder {
    async fn id(&mut self, token: &str) -> Result<String, String> {
        Ok(match self {
            Self::Name(folder_name) => {
                let drive_folder = create_folder(token, folder_name).await?;
                let id = drive_folder.id.clone();
                *self = Self::Info(drive_folder);
                id
            }
            Self::Info(drive_folder) => drive_folder.id.clone(),
        })
    }
}

impl Default for AppFolder {
    fn default() -> Self {
        Self::Name("__@@md-viewer@@__")
    }
}

#[derive(Debug, Default)]
pub struct DriveManager {
    app_folder: Mutex<AppFolder>,
}

impl DriveManager {
    pub async fn app_folder_id(&self, token: &str) -> Result<String, String> {
        unlock(&self.app_folder, "app folder id")?.id(token).await
    }
}
