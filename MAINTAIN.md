# Upgrade Dependencies
Run:
```bash
cargo upgrade && cargo update
```

# Testing and Linting
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
Update `Cargo.toml`:
```toml
version = "3.0.0"
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