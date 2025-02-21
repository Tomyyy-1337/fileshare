use std::{collections::HashMap, path::PathBuf, sync::{Arc, RwLock}};

#[derive(Debug, Clone)]
pub struct FileInfo {
    pub path: PathBuf,
    pub size: usize,
    pub download_count: usize,
}

pub struct FileManager {
    paths: Arc<RwLock<HashMap<usize, FileInfo>>>,
    view: Vec<(usize, FileInfo)>,
    index: usize,
}

impl FileManager {
    pub fn new() -> Self {
        Self {
            paths: Arc::new(RwLock::new(HashMap::new())),
            view: Vec::new(),
            index: 0,
        }
    }

    pub fn increment_download_count(&mut self, index: usize) {
        if let Some(file) = self.view.iter_mut().find(|(i, _)| *i == index) {
            file.1.download_count += 1;
            self.paths.write().unwrap().get_mut(&index).unwrap().download_count += 1;
        }
    }

    pub fn get_arc(&self) -> Arc<RwLock<HashMap<usize, FileInfo>>> {
        self.paths.clone()
    }

    pub fn get_view(&self) -> &Vec<(usize, FileInfo)> {
        &self.view
    }

    pub fn get(&self, index: usize) -> Option<FileInfo> {
        self.view.iter().find(|(i, _)| *i == index).map(|(_, file)| file.clone())
    }

    pub fn push(&mut self, path: PathBuf, size: usize) {
        let file = FileInfo {
            path,
            size,
            download_count: 0,
        };
        self.paths.write().unwrap().insert(self.index, file.clone());
        self.view.push((self.index, file));
        self.index += 1;
    }

    pub fn remove(&mut self, index: usize) {
        self.paths.write().unwrap().remove(&index);
        self.view.retain(|(i, _)| *i != index);
    }

    pub fn clear(&mut self) {
        self.paths.write().unwrap().clear();
        self.view.clear();
    }
}