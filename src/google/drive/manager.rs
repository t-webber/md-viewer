use super::interface::DriveFile;
use crate::google::drive::interface::{FileType, create_folder, root_contains_file};
use crate::log;

#[derive(Debug)]
enum AppFolder {
    Name(String),
    Info(DriveFile),
}

impl AppFolder {
    const fn inner(&self) -> &Self {
        self
    }
}

#[derive(Debug)]
pub struct DriveManager {
    app_folder: async_lock::Mutex<AppFolder>,
}

impl DriveManager {
    pub const fn new(folder_name: String) -> Self {
        Self { app_folder: async_lock::Mutex::new(AppFolder::Name(folder_name)) }
    }

    pub async fn app_folder_id(&self, token: &str) -> Result<Box<str>, String> {
        let mut app_folder = self.app_folder.lock().await;
        Ok(match app_folder.inner() {
            AppFolder::Info(folder) => folder.to_id(),
            AppFolder::Name(name) => {
                log!("App folder id not loaded");
                let folder = if let Some(folder) =
                    root_contains_file(token, name, &FileType::Folder).await?
                {
                    folder
                } else {
                    log!("App folder doesn't exist");
                    create_folder(token, name).await?
                };
                let id = folder.to_id();
                *app_folder = AppFolder::Info(folder);
                id
            }
        })
    }
}
