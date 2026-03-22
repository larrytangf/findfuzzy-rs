fuzzy find tool to replace `fd` and `fzf` in Rust

## Usage Examples

### Find Files
```bash
# Find all files containing "test"
cargo run -- find test

# Find only files (not directories)
cargo run -- find "*.rs" --files-only

# Case-insensitive search with depth limit
cargo run -- find config -i -D 3

# Search in specific directory
cargo run -- find log --path /var/log
```

### Fuzzy Search
```bash
# Fuzzy search from stdin
echo -e "hello\nworld\nhello_world" | cargo run -- fzf hlo

# Limit results
echo -e "apple\napricot\nbanana" | cargo run -- fzf ap -n 5
```

### Combined Search
```bash
# Find all .rs files and fuzzy search
cargo run -- search "main" --path . -D 5
```

## Key Features

✨ **Features Implemented:**
- **File Discovery**: Recursive directory walking with pattern matching
- **Fuzzy Matching**: Smart fuzzy search algorithm with scoring
- **Case-Insensitive**: Optional case-insensitive matching
- **Depth Control**: Limit search depth to avoid deep recursion
- **Filtering**: Search for files only, directories only, or both
- **Performance**: Efficient string matching and sorting
- **CLI Interface**: Clean command-line argument parsing with `clap`

## Performance Tips

- Use `--max-depth` to limit recursion for large directories
- Combine `--files-only` with specific patterns for faster searches
- Use case-insensitive mode (`-i`) for more flexible matching

This tool provides a solid foundation that you can extend with additional features like regex support, colored output, or interactive selection!
