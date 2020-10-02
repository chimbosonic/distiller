extern crate walkdir;
extern crate serde;
extern crate comment_parser;
extern crate sha3;
extern crate serde_rusqlite;
extern crate rusqlite;
extern crate env_logger;
extern crate clap;
extern crate num_cpus;
extern crate threadpool;

use clap::{Arg, App};
use walkdir::WalkDir;
use std::path::{Path,PathBuf};
use comment_parser::CommentParser;
use std::fs;
use std::ffi::OsStr;
use sha3::{Sha3_512, Digest};
use serde::{Deserialize, Serialize};
use serde_rusqlite::*;
use rusqlite::{NO_PARAMS , params, Connection, Transaction};
use threadpool::ThreadPool;
use std::sync::mpsc::channel;

#[derive(Serialize, Deserialize)]
struct Comment {
	comment: String, // String containing the comment
	hash: String, // Hash for the comment
	filehash: String,// Hash of the file where the comment was found
}

#[derive(Serialize, Deserialize)]
struct FileData {
	path: String, // Path of the file
	hash: String, // Sha3_512 hash of the file
	comments: Vec<Comment>, // Vector of Comments
}

fn main() {
	let matches = App::new("distiller")
		.version("2.0.0")
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
	connection.close().unwrap();
	search_source(scandir.to_string(), dbpath.to_string());
	println!("Done");
}

/*
	Reads a file and returns a FileContents struct containing:
		- contents: String containing all file contents but replacing non UTF8 chars
		- hash: Hash of the file
*/
fn read_file(filepath: &Path) -> Result<(String, String)> {
	log::info!("Reading {}",filepath.display().to_string());

	let buf = fs::read(filepath).unwrap();
	let contents = String::from_utf8_lossy(&buf).into_owned().to_string();
	let mut hasher = Sha3_512::new();
	hasher.update(&buf);
	let hash = hasher.finalize();

	Ok((contents.to_owned(), format!("{:x}", hash).to_owned()))
}


/*
	For a given path and sqlite connection
	This will walk the given path and then look for comments in every file with an extension of (.c,.cpp,.cxx,.h,.rs).
	Comments are then store in the sqlite db
*/
fn search_source(path: String, dbpath: String) {
	log::info!("Searching {} for comments",path);

	let pool = ThreadPool::new(num_cpus::get());
	let (tx, rx) = channel();

	for e in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
		let metadata = e.metadata().unwrap();
		let path = e.path();
		if metadata.is_file() {

			let extension = path.extension();
			
			if Some(OsStr::new("c")) == extension || Some(OsStr::new("cpp")) == extension || Some(OsStr::new("cxx")) == extension || Some(OsStr::new("h")) == extension || Some(OsStr::new("rs")) == extension {
				let p = path.to_owned();
				let tx = tx.clone();

				pool.execute(move || {
					let filedata = get_comments(p);
					tx.send(filedata).expect("Could not send data!");
				});

			}
		}
	}

	drop(tx);
	
	let mut datas = Vec::<FileData>::new();
	for t in rx.iter() {
		let filedata = t;
		datas.push(filedata);	
	}

	write_to_db(datas,dbpath);

}

fn write_to_db(datas: Vec<FileData>,dbpath: String){
	let mut connection = rusqlite::Connection::open(&dbpath).unwrap();
	let transaction = connection.transaction().unwrap();
	for data in datas{
		add_filedata_sql(&data,&transaction);
	}
	log::info!("Commiting to Database");
	transaction.commit().unwrap();
	connection.close().unwrap();
}



/*
	Extracts Comments from a given file
	Creating a FileData struct containing:
		- Comments: Vector of Comments
		- path: String containing the relative path to the file
		- hash: Sha3-512 hash of the file
	A Comment struct contains:
		- filehash: Sha3-512 hash of the file
		- hash: Sha3-512 hash of the comment
		- comment: String containing the extracted comment
*/
fn get_comments(path: PathBuf) -> FileData {
	log::info!("Extracting comments from: {}", path.display());

	let comment_parser_rules = comment_parser::get_syntax_from_path(&path).unwrap();
	let (filecontents,filehash) = read_file(&path).unwrap();
	let parser = CommentParser::new(&filecontents, comment_parser_rules);
	let filepath = path.display().to_string();
	let mut comments = Vec::<Comment>::new();
	for comment in parser {
		if comment.text().to_string().len() >= 5 {
			let mut hasher = Sha3_512::new();
			hasher.update(comment.text().to_string());
			let commenthash = hasher.finalize();
			let commentdata = Comment {
				comment: comment.text().to_string().to_owned(),
				hash:	 format!("{:x}", commenthash).to_owned(),
				filehash: filehash.to_owned(),
			};
			comments.push(commentdata);
		}	
	}

	let filedata = FileData {
			comments: comments,
			path: filepath.to_owned(),
			hash: filehash.to_owned(),
	};
	
	log::info!("Extracted comments from: {}", &filedata.path);
	return filedata;
}

/*
	Adds a given filedata struct to the sqlite 3 database
*/
fn add_filedata_sql(filedata: &FileData, transaction: &Transaction) {
	log::info!("Adding {}'s results to results database", filedata.path);
	for comment in filedata.comments.as_slice() {
		transaction.execute_named("INSERT INTO comments (id, comment, filehash) VALUES (:hash, :comment, :filehash)", &to_params_named(&comment).unwrap().to_slice()).unwrap();
	}
	transaction.execute("INSERT INTO files (id, filename) VALUES (?1, ?2)", params![filedata.hash, filedata.path],).unwrap();
}

/*
Creates Tables:
	comments containing:
		id: hash of the comment
		comment: the comment
		filehash: hash of the file where comment came from
	files contiaining:
		id: hash of the file
		filename: path of the file
*/
fn create_table(connection: &Connection) {
	connection.execute("CREATE TABLE comments (id TEXT, comment TEXT, filehash TEXT)", NO_PARAMS).unwrap();
	connection.execute("CREATE TABLE files (id TEXT, filename TEXT)", NO_PARAMS).unwrap();
}
