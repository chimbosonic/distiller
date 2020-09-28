extern crate walkdir;
extern crate serde;
extern crate comment_parser;
extern crate sha3;
extern crate uuid;
extern crate serde_rusqlite;
extern crate rusqlite;
extern crate env_logger;
extern crate serde_json;
extern crate clap;

use clap::{Arg, App};
use walkdir::WalkDir;
use std::path::Path;
use comment_parser::CommentParser;
use std::fs;
use std::ffi::OsStr;
use sha3::{Sha3_512, Digest};
use serde::{Deserialize, Serialize};
use serde_rusqlite::*;
use uuid::Uuid;
use rusqlite::{NO_PARAMS , params, Connection};


#[derive(Serialize, Deserialize)]
struct Comment {
	comment: String, // String containing the comment
	uuid: Uuid, // uuid for the comment
	fileid: Uuid,// uuid of the file where the comment was found
}

#[derive(Serialize, Deserialize)]
struct FileData {
	path: String, // Path of the file
	hash: String, // Sha3_512 hash of the file
	uuid: Uuid, // uuid for the file
	comments: Vec<Comment>, // Vector of Comments
}

fn main() {
	let matches = App::new("distiller")
		.version("1.0.0")
		.about("Extracts all comments in source code to sqlite db")
		.arg(Arg::with_name("output")
			.short("o")
			.long("output")
			.value_name("FILE")
			.help("Sets the output db file defaults to results.db")
			.takes_value(true))
		.arg(Arg::with_name("INPUT")
			.short("i")
			.long("input")
			.help("Sets the source directory to parse")
			.value_name("DIRECTORY")
			.required(true)
			.takes_value(true))
		.get_matches();
	let dbpath = matches.value_of("output").unwrap_or("results.db");
	let scandir = matches.value_of("INPUT").unwrap();
	
	env_logger::init(); //Setup logging Make sure to use RUST_LOG=info
	fs::remove_file(&dbpath).ok(); //Remove any existing results database
	let connection = rusqlite::Connection::open(&dbpath).unwrap();
	create_table(&connection);
	search_source(scandir.to_string(), connection);
}

/*
	For a given path and sqlite connection
	This will walk the given path and then look for comments in every file with an extension of (.c,.cpp,.cxx,.h,.rs).
	Comments are then store in the sqlite db
*/
fn search_source(path: String, connection: Connection){
	log::info!("Searching {} for comments",path);

	for e in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
		let metadata = e.metadata().unwrap();
		let path = e.path();
		if metadata.is_file() {
			let extension = path.extension();

			if Some(OsStr::new("c")) == extension || Some(OsStr::new("cpp")) == extension || Some(OsStr::new("cxx")) == extension || Some(OsStr::new("h")) == extension {
				let filedata = get_comments(&path);
				if !filedata.comments.is_empty(){
					// write_filedata(&filedata);
					add_filedata_sql(&filedata,&connection);
				}
			}
		}
	}
}

/*
	Extracts Comments from a given file
	Creating a FileData struct containing:
		- Comments: Vector of Comments
		- path: String containing the relative path to the file
		- hash: Sha3-512 hash of the file
		- uuid: Uuid V4 for the file
	A Comment struct contains:
		- fileid: Uuid V4 for the file (same as uuid found in FileData)
		- uuid: Uuid V4 for the comment
		- comment: String containing the extracted comment
*/
fn get_comments(path: &Path) -> FileData {
	log::info!("Extracting comments from: {}", path.display());

	let comment_parser_rules = comment_parser::get_syntax_from_path(path).unwrap();
	let filecontents = read_file(&path);
	let parser = CommentParser::new(&filecontents.contents, comment_parser_rules);
	let filehash = &filecontents.hash;
	let filepath = path.display().to_string();
	let mut comments = Vec::<Comment>::new();
	let fileid = Uuid::new_v4();
	for comment in parser {
		if comment.text().to_string().len() >= 5 {
			let commentdata = Comment {
				comment: comment.text().to_string().to_owned(),
				uuid:	 Uuid::new_v4(),
				fileid: fileid,
			};
			comments.push(commentdata);
		}	
	}

	let filedata = FileData {
			comments: comments,
			path: filepath.to_owned(),
			hash: filehash.to_owned(),
			uuid: fileid,
	};
	
	log::info!("Extracted comments from: {}", &filedata.path);
	return filedata;
}

struct FileContents {
	contents: String, // String containing contents
	hash: String, // Hash of the file
}

/*
	Reads a file and returns a FileContents struct containing:
		- contents: String containing all file contents but replacing non UTF8 chars
		- hash: Hash of the file
*/
fn read_file(path: &Path) -> FileContents {
	let buf = fs::read(path).unwrap();
	let contents = String::from_utf8_lossy(&buf).into_owned().to_string();
	let mut hasher = Sha3_512::new();
	hasher.update(&buf);
	let hash = hasher.finalize();
	let filecontents = FileContents {
		contents: contents.to_owned(),
		hash: format!("{:x}", hash).to_owned(),
	};
	return filecontents
}


fn add_filedata_sql(filedata: &FileData, connection: &Connection) {
	for comment in filedata.comments.as_slice() {
		connection.execute_named("INSERT INTO comments (id, comment, fileid) VALUES (:uuid, :comment, :fileid)", &to_params_named(&comment).unwrap().to_slice()).unwrap();
	}
	connection.execute("INSERT INTO files (id, filename, filehash) VALUES (?1, ?2, ?3)", params![filedata.uuid.to_string(), filedata.path, filedata.hash],).unwrap();
}

fn create_table(connection: &Connection) {
	connection.execute("CREATE TABLE comments (id TEXT, comment TEXT, fileid TEXT)", NO_PARAMS).unwrap();
	connection.execute("CREATE TABLE files (id TEXT, filename TEXT, filehash TEXT)", NO_PARAMS).unwrap();
}
