use std::{
    error::Error,
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_512};

use comment_parser::CommentParser;
use std::ffi::OsStr;
use std::sync::mpsc::channel;
use threadpool::ThreadPool;
use walkdir::WalkDir;

use crate::db;

pub type Result<T> = std::result::Result<T, Box<dyn Error + Send + Sync>>;

#[derive(Serialize, Deserialize)]
pub struct FileData {
    pub path: String,           // Path of the file
    pub hash: String,           // Sha3_512 hash of the file
    pub comments: Vec<Comment>, // Vector of Comments
}

#[derive(Serialize, Deserialize)]
pub struct Comment {
    pub comment: String,
    pub hash: String,
    pub file_hash: String,
}

impl FileData {
    fn get_hash(buf: &[u8]) -> String {
        let mut hasher = Sha3_512::new();
        hasher.update(buf);
        let hash = hasher.finalize();

        format!("{:x}", hash)
    }

    fn read_file(filepath: &Path) -> Result<(String, String)> {
        log::info!("Reading {}", filepath.display().to_string());

        let buf = fs::read(filepath)?;
        let contents = String::from_utf8_lossy(&buf).into_owned().to_string();
        let hash = Self::get_hash(&buf);

        Ok((contents, hash))
    }

    fn get_comments(
        file_contents: &str,
        file_path: &Path,
        file_hash: &String,
    ) -> Result<Vec<Comment>> {
        log::info!(
            "Extracting comments from: {}",
            file_path.display().to_string()
        );

        let comment_parser_rules = match comment_parser::get_syntax_from_path(file_path) {
            Ok(syntax) => syntax,
            Err(_) => return Err("Could not get syntax".into()),
        };
        let parser = CommentParser::new(file_contents, comment_parser_rules);

        let mut comments = Vec::<Comment>::new();

        for comment in parser {
            if comment.text().to_string().len() >= 5 {
                let commentdata = Comment {
                    comment: comment.text().to_string(),
                    hash: Self::get_hash(comment.text().as_bytes()),
                    file_hash: file_hash.to_string(),
                };
                comments.push(commentdata);
            }
        }

        Ok(comments)
    }

    pub fn new(file_path: PathBuf) -> Result<FileData> {
        let path = file_path.display().to_string();
        let (file_contents, file_hash) = match Self::read_file(&file_path) {
            Ok((contents, hash)) => (contents, hash),
            Err(_) => {
                log::error!("Could not read file: {}", &path);
                return Err("Could not read file".into());
            }
        };

        let comments = match Self::get_comments(&file_contents, &file_path, &file_hash) {
            Ok(comments) => comments,
            Err(_) => {
                log::error!("Could not get comments from: {}", &path);
                return Err("Could not get comments".into());
            }
        };

        let filedata = FileData {
            comments,
            path,
            hash: file_hash,
        };

        log::info!("Extracted comments from: {}", &filedata.path);
        Ok(filedata)
    }
}

pub fn search_source(path: String, dbpath: String) -> Result<()> {
    log::info!("Searching {} for comments", path);

    let pool = ThreadPool::new(num_cpus::get());
    let (tx, rx) = channel();

    for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
        let metadata = match entry.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };

        if !metadata.is_file() {
            continue;
        }

        let path = entry.path();
        let extension = match path.extension() {
            Some(extension) => extension,
            None => continue,
        };

        if !is_supported_extension(extension) {
            continue;
        }

        let path_buf = path.to_owned();
        let tx = tx.clone();

        pool.execute(move || {
            let filedata = FileData::new(path_buf);
            tx.send(filedata).expect("Could not send data!");
        });
    }

    drop(tx);

    let mut file_datas = Vec::<FileData>::new();
    for file_data in rx.iter() {
        match file_data {
            Ok(file_data) => file_datas.push(file_data),
            Err(err) => {
                log::error!("Could not get file data: {}", err);
                continue;
            }
        }
    }

    db::write_to_db(file_datas, dbpath)?;

    Ok(())
}

fn is_supported_extension(extension: &OsStr) -> bool {
    OsStr::new("c") == extension
        || OsStr::new("cpp") == extension
        || OsStr::new("cxx") == extension
        || OsStr::new("h") == extension
        || OsStr::new("rs") == extension
}
