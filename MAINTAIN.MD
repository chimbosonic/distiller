# Upgrade Dependencies
Run:
```bash
cargo upgrade && cargo update
```

# Testing and Linting
## License checks and Security Advisory checks
Use [cargo-deny](https://github.com/EmbarkStudios/cargo-deny):
```bash
cargo deny check
```

## Test the code
Create a DB of the test_files
```bash
cargo run --release -- -i test_files
```
Then check the DB file with `sqlite3`
```bash
sqlite3 results.db
```
List tables:
```sqlite3
.tables
```
List contents of comments:
```sqlite3
SELECT * FROM comments
```
List contents of files:
```sqlite3
SELECT * FROM files
```
Exit sqlite3:
```sqlite3
.exit
```
# Version Bump
Update version number in `README.md`:
```markdown
## Usage

bash
distiller --help
distiller 2.0.3
Extracts all comments in source code to sqlite db
....
```

Update version number found when running `--help` in `main.rs` :
```rust
let matches = App::new("distiller")
    .version("2.0.3")
```

Also update `Cargo.toml`:
```toml
version = "2.0.3"
```

Update `Cargo.lock`:
```bash
cargo update
```

Once all this is done commit and create a git tag with :
```bash
git tag v<NUMBER>
```

Push code to github.com:
```bash
git push
```

Create a release on Github which will trigger the publish action