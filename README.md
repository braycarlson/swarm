# swarm

swarm is a developer tool for generating project context.

<img src="assets/screenshot.png?raw=true" alt="A screenshot demonstrating the capabilities of swarm">

## Search Filters

| Filter | Syntax | Shorthand | Description | Example |
|--------|--------|-----------|-------------|---------|
| **Contains** | `term` | - | Matches filename or path containing term | `utils` |
| **Exact Match** | `"term"` | - | Matches exact filename | `"main.rs"` |
| **Exclude** | `-term` or `!term` | - | Excludes files/folders containing term | `-test` |
| **Extension** | `ext:value` | `e:value` | Filter by file extension (comma-separated) | `ext:rs,toml` |
| **Exclude Extension** | `-ext:value` | `-e:value` | Exclude by file extension | `-ext:log` |
| **Name** | `name:value` | `n:value` | Filter by filename (comma-separated) | `name:mod,lib` |
| **Exclude Name** | `-name:value` | `-n:value` | Exclude by filename | `-name:test` |
| **Path** | `path:value` | `p:value` | Filter by path (comma-separated) | `path:src/app` |
| **Exclude Path** | `-path:value` | `-p:value` | Exclude by path | `-path:vendor` |
| **Type** | `type:value` | `t:value` | Filter by type: `file`/`f` or `dir`/`directory`/`d`/`folder` | `type:file` |
| **Depth** | `depth:value` | `d:value` | Maximum directory depth (supports `<=`, `<`) | `depth:<=2` |
| **Size** | `size:value` | `s:value` | Filter by file size (supports `>=`, `>`, `<=`, `<`, ranges) | `size:>1kb` |
| **Lines** | `lines:value` | `l:value` | Filter by line count (supports `>=`, `>`, `<=`, `<`, ranges) | `lines:100-500` |
| **Recent** | `recent:value` | `r:value` | Filter by modification time | `recent:1d` |
| **Content** | `content:value` | `c:value` | Search within file contents | `content:TODO` |
| **Git Status** | `git:value` | `g:value` | Filter by git status (comma-separated) | `git:m,u` |
| **Exclude Git** | `-git:value` | `-g:value` | Exclude by git status | `-git:u` |

### Commands

| Command | Aliases | Description | Example |
|---------|---------|-------------|---------|
| `--diff` | `--d` | Include original and modified versions of changed files | `--diff` |
| `--plain` | `--plain-text`, `--text` | Output as plain text (overrides options) | `--plain` |
| `--markdown` | `--md` | Output as Markdown (overrides options) | `--markdown` |
| `--json` | - | Output as JSON (overrides options) | `--json` |
| `--xml` | - | Output as XML (overrides options) | `--xml` |

### Size Units

| Unit | Description |
|------|-------------|
| `b` | Bytes |
| `kb` | Kilobytes |
| `mb` | Megabytes |
| `gb` | Gigabytes |

### Time Units

| Unit | Description |
|------|-------------|
| `m` | Minutes |
| `h` | Hours |
| `d` | Days |
| `w` | Weeks |
| `today` | Last 24 hours |

### Git Status Values

| Value | Shorthand | Description |
|-------|-----------|-------------|
| `added` | `a` | Newly added files |
| `modified` | `m` | Modified files |
| `deleted` | `d` | Deleted files |
| `renamed` | `r` | Renamed files |
| `staged` | `s` | Staged files |
| `untracked` | `u` | Untracked files |
| `conflicted` | `x` | Files with merge conflicts |
| `changed` | `c` | Any file with a diff |

### Examples
```
ext:rs path:src                    # Rust files in src directory
ext:rs,toml -path:target           # Rust and TOML files, excluding target
git:m,u                            # Modified and untracked files
size:>10kb lines:<1000             # Files larger than 10KB with fewer than 1000 lines
recent:1w ext:rs                   # Rust files modified in the last week
type:dir depth:<=2                 # Directories at depth 2 or less
content:TODO ext:rs                # Rust files containing "TODO"
--diff git:m                       # Show diffs for modified files
--json ext:rs                      # Output Rust files as JSON
--markdown path:src                # Output files in src as Markdown
"Cargo.toml"                       # Exact match for Cargo.toml
```
