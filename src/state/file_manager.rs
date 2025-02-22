use std::{collections::HashMap, path::{Path, PathBuf}, sync::{Arc, RwLock}};
use std::{fs::File, io::{Read, Write}};

use futures::SinkExt;
use tokio::task::{self, yield_now};
use walkdir::WalkDir;
use zip::write::SimpleFileOptions;
use iced::task::Handle;

pub const TEMP_DIR: &str = ".\\temp";

#[derive(Debug, Clone)]
pub enum ZipMessage {
    Done{path: PathBuf},
    Started{path: PathBuf, num_files: usize},
    Progress{path: PathBuf}, 
}

#[derive(Debug, Clone)]
pub struct CompressingZip {
    pub num_files: usize,
    pub progress: usize,
    pub handle: Handle,
    pub start_time: std::time::Instant,
}

#[derive(Debug, Clone)]
pub struct FileInfo {
    pub path: PathBuf,
    pub size: usize,
    pub download_count: usize,
    pub is_zip: bool,
}

pub struct FileManager {
    paths: Arc<RwLock<HashMap<usize, FileInfo>>>,
    view: Vec<(usize, FileInfo)>,
    index: usize,
    compressing_zips: HashMap<PathBuf, CompressingZip>,
}

impl FileManager {
    pub fn new() -> Self {
        Self {
            paths: Arc::new(RwLock::new(HashMap::new())),
            view: Vec::new(),
            index: 0,
            compressing_zips: HashMap::new(),
        }
    }

    
    fn path_to_zip_path(path: &Path) -> PathBuf {
        let file_name = path.file_name().unwrap().to_str().unwrap();
        Path::new(TEMP_DIR).join(file_name).with_extension("zip")
    }

    pub fn add_new_zip_compressing(&mut self, path: PathBuf, handle: Handle) {
        self.compressing_zips.insert(path, CompressingZip {
            num_files: 0,
            progress: 0,
            handle,
            start_time: std::time::Instant::now(),
        });
    }

    pub fn set_zip_num_files(&mut self, path: &PathBuf, num_files: usize) {
        if let Some(zip) = self.compressing_zips.get_mut(path) {
            zip.num_files = num_files;
        }
    }

    pub fn update_zip_compressing(&mut self, path: &PathBuf) {
        if let Some(zip) = self.compressing_zips.get_mut(path) {
            zip.progress += 1;
        }
    }

    pub fn zip_compressing_canceld(&mut self, path: &PathBuf) {
        let info = self.compressing_zips.remove(path);
        if let Some(info) = info {
            let _ = std::fs::remove_file(Self::path_to_zip_path(path));
            info.handle.abort();
        }
    }

    pub fn already_compressed(&self, path: &PathBuf) -> bool {
        self.compressing_zips.contains_key(path) ||
        self.view.iter().any(|(_, file)| file.path.file_name().unwrap() == PathBuf::from(TEMP_DIR).join(path.file_name().unwrap()).with_extension("zip").file_name().unwrap())
    }

    pub fn zip_compressing_done(&mut self, path: &PathBuf) {
        self.compressing_zips.remove(path);

        let write_dir = Self::path_to_zip_path(path);

        if !self.view.iter().any(|(_, file)| file.path == write_dir) {
            self.push(write_dir, true);
        }
    }

    pub fn get_zip_compressing(&self) -> Vec<(&PathBuf, &CompressingZip)> {
        let mut result = self.compressing_zips
            .iter()
            .map(|(p, z)| (p, z))
            .collect::<Vec<_>>();

        result.sort_by(|(_, a), (_, b)| a.start_time.cmp(&b.start_time));
    
        result
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

    pub fn push(&mut self, path: PathBuf, is_zip: bool) {
        let size = path.metadata().unwrap().len() as usize;
        let file = FileInfo {
            path,
            size,
            download_count: 0,
            is_zip,
        };
        self.paths.write().unwrap().insert(self.index, file.clone());
        self.view.push((self.index, file));
        self.index += 1;
    }

    pub fn remove(&mut self, index: usize) {
        if let Some(file) = self.view.iter().find(|(i, _)| *i == index) {
            if file.1.is_zip {
                let _ = std::fs::remove_file(&file.1.path);
            }
        }

        self.paths.write().unwrap().remove(&index);
        self.view.retain(|(i, _)| *i != index);
    }

    pub fn clear(&mut self) {
        for (_, file) in self.view.iter() {
            if file.is_zip {
                let _ = std::fs::remove_file(&file.path);
            }
        }

        self.paths.write().unwrap().clear();
        self.view.clear();
    }

    pub async fn zip_task(path: PathBuf, mut tx: futures::channel::mpsc::Sender<ZipMessage>) {
        let dst_path = Self::path_to_zip_path(&path);
        let _ = std::fs::create_dir_all(dst_path.parent().unwrap());
        let file = File::create(&dst_path).unwrap();

        let walkdir = WalkDir::new(&path);
        let it = walkdir.into_iter();
        let it = &mut it.filter_map(|e| e.ok());

        let mut zip = zip::ZipWriter::new(file);
        let options = SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated)
            .large_file(true)
            .unix_permissions(0o755);

        let prefix = Path::new(&path);
        let mut buffer = Vec::new();
        let it: Vec<_> = it.collect();
        let number_of_files = it.len();
        for entry in it {
            let path = entry.path();
            let _ = tx.send(ZipMessage::Progress{path: prefix.to_path_buf()}).await;
            let name = path.strip_prefix(prefix).unwrap();
            let path_as_string = name
            .to_str()
            .map(str::to_owned)
            .unwrap();
        
            if path.is_file() {
                zip.start_file(path_as_string, options).unwrap();
                let mut f = File::open(path).unwrap();
                f.read_to_end(&mut buffer).unwrap();
                zip.write_all(&buffer).unwrap();
                buffer.clear();
            } else if !name.as_os_str().is_empty() {
                zip.add_directory(path_as_string, options).unwrap();
            }
            let _ = tx.send(ZipMessage::Started{path: prefix.to_path_buf(), num_files: number_of_files}).await;
            yield_now().await;
        }
        let _ = zip.finish();
        let _ = tx.send(ZipMessage::Done{path: prefix.to_path_buf()}).await;
    }


    pub async fn start_zip_task(path: PathBuf, tx: futures::channel::mpsc::Sender<ZipMessage>) {
        let handle = task::spawn(async move {
            Self::zip_task(path, tx).await;
        });

        let _ = handle.await;
    }
}