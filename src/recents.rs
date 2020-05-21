use gio::prelude::*;
use std::path::{Path, PathBuf};

use crate::object_wrapper::ObjectWrapper;
use crate::settings::{Key, SettingsManager};

#[derive(Clone, Serialize, Deserialize)]
pub struct RecentItem {
    pub path: PathBuf,
}

pub struct RecentManager {
    pub model: gio::ListStore,
}

impl RecentManager {
    pub fn new() -> Self {
        let model = gio::ListStore::new(ObjectWrapper::static_type());

        let recents_manager = Self { model };

        recents_manager.init();
        recents_manager
    }

    fn init(&self) {
        let history = RecentManager::get_history();
        for item in history {
            let object = ObjectWrapper::new(Box::new(RecentItem { path: item }));
            self.model.insert(0, &object);
        }
    }

    pub fn get_history() -> Vec<PathBuf> {
        // As gtk-rs doesn't have bindings for array gvariants
        // we will store the history in gsettings as a comma sperated string instead
        // Hopefully we can ditch this in the future.
        let recent_files = SettingsManager::get_string(Key::RecentFiles);

        recent_files.split(';').map(|file| Path::new(file).to_path_buf()).filter(|path| path.exists()).collect::<Vec<PathBuf>>()
    }

    pub fn store(history: Vec<PathBuf>) {
        let recent_files = history.iter().map(|path| path.to_str().unwrap()).collect::<Vec<&str>>().join(";");

        SettingsManager::set_string(Key::RecentFiles, recent_files);
    }

    fn index(&self, item: &RecentItem) -> Option<u32> {
        for i in 0..self.model.get_n_items() {
            let gobject = self.model.get_object(i).unwrap();
            let a: RecentItem = gobject.downcast_ref::<ObjectWrapper>().unwrap().deserialize();

            if item.path == a.path {
                return Some(i);
            }
        }
        None
    }

    pub fn add(&self, path: PathBuf) {
        if path.exists() {
            let mut history = RecentManager::get_history();
            let item = RecentItem { path };

            if history.contains(&item.path) {
                history.retain(|p| p != &item.path); // Delete the old elements
            }

            if let Some(index) = self.index(&item) {
                self.model.remove(index);
            }
            history.push(item.path.clone());
            self.model.insert(0, &ObjectWrapper::new(Box::new(item)));

            RecentManager::store(history);
        }
    }
}
