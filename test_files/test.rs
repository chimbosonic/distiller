extern crate walkdir;
extern crate serde_json;
extern crate serde;
extern crate comment_parser;
extern crate sha3;
extern crate uuid;

use walkdir::WalkDir;
use comment_parser::CommentParser;
use std::fs::File;
use std::fs;
use std::io::prelude::*;
use std::ffi::OsStr;
use sha3::{Sha3_512, Digest};
// use serde_json::json;
use serde::{Deserialize, Serialize};
use serde_json::Result;
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
struct Comment {
	comment: String,
	uuid: String,
}

#[derive(Serialize, Deserialize)]
struct FileData {
	filepath: String,
	filehash: String,
	comments: Vec<Comment>,
}

fn main() {
	for e in WalkDir::new(".").into_iter().filter_map(|e| e.ok()) {
			if e.metadata().unwrap().is_file() {
				println!("{}", e.path().display());
				let extension = e.path().extension();
				if Some(OsStr::new("c")) == extension || Some(OsStr::new("cpp")) == extension || Some(OsStr::new("cxx")) == extension || Some(OsStr::new("h")) == extension {
					let rules = comment_parser::get_syntax_from_path(e.path()).unwrap();
					let mut file = File::open(e.path()).unwrap();
					let mut buf = vec![];
					file.read_to_end (&mut buf).unwrap();
					let contents = String::from_utf8_lossy(&buf).into_owned().to_string();
					let parser = CommentParser::new(&contents, rules);
					let filehash = get_hash(&contents);
					let filepath = e.path().display().to_string();
					let mut comments = Vec::<Comment>::new();

					for comment in parser {
						if comment.text().to_string().len() >= 5 {
							let uuid = Uuid::new_v4().to_string();
							let comment_string = comment.text().to_string();
	
							let commentdata = Comment {
								comment: comment_string.to_owned(),
								uuid:	uuid.to_owned(),
							};
							comments.push(commentdata);
						}	
					}
					if !comments.is_empty() {
						let filedata = FileData {
							comments: comments,
							filepath: filepath.to_owned(),
							filehash: filehash.to_owned(),
						};
						// print_filedata(&filedata).unwrap();
						write_filedata(&filedata).unwrap();
					}
				}
		}
	}
}


fn get_hash(data: &String) -> String {
	let mut hasher = Sha3_512::new();
	hasher.update(data);
	let hash = hasher.finalize();
	return format!("{:x}", hash); 
}


fn print_filedata(filedata: &FileData) -> Result<()> {
	let j = serde_json::to_string_pretty(filedata)?;
	println!("{}", j);
	Ok(())
}

fn write_filedata(filedata: &FileData) -> std::io::Result<()> {
	fs::create_dir_all("./results/")?;
	let filepath = format!("./results/{}.json",Uuid::new_v4().to_string());
	let j = serde_json::to_string(filedata)?;
	let mut file = File::create(filepath)?;
	file.write_all(j.as_bytes())?;
	Ok(())
}