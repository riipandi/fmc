> Point-in-time research, 2026-11-26. Source: agent research for plan-00001 (Scouter at commit `434d000f0f934fb04fced8b1bc1d0e673c6ed4fe`). `scouter:<path>` is relative to the Scouter repository at that commit. Other scratch paths (`<scratch>/…`) were local working copies and probes that are not kept. The tree and the locked crates win over this file.

# sqltui reference study

Subject: https://github.com/dzulfikar08/sqltui (cloned to `<scratch>/sqltui`, shallow clone at commit `90bcf39`, "Prepare repo for public collaboration").
Purpose: input for a post-parity fmc milestone (SQLite and DuckDB file viewer on `opentui` 0.9.9).
Method: read-only source review. Nothing was built, installed or executed. Paths below are relative to `<scratch>/sqltui` unless absolute.
Crate data: crates.io API and docs.rs, fetched 2026-11-26.

## 1. What it is

- Rust terminal SQLite browser, package `sqltui` 3.1.0, GPL-3.0 (`Cargo.toml:2-6`, `LICENSE`).
- Two front ends in one binary: a ratatui TUI (default) and an axum HTTP server with a React SPA (`--ui`, `src/main.rs:29-31`, `src/api.rs`).
- Only SQLite. No DuckDB anywhere: no `duckdb` in `Cargo.toml`, `Cargo.lock`, or sources (grep clean).
- Source size: about 2,650 lines of Rust (`src/ui/talbe_view.rs` is 1,433 of them) plus a 642-line `web/src/App.tsx`.
- No tests (`AGENTS.md:10`, `AGENTS.md:69`).

## 2. Stack and UI library

| Concern | Choice | Evidence |
|---|---|---|
| Language | Rust, edition 2021 | `Cargo.toml:4` |
| TUI | ratatui 0.29.0 + crossterm 0.28.1 | `Cargo.toml:19,21` |
| SQLite | rusqlite 0.33.0 (lock: libsqlite3-sys 0.31.0) | `Cargo.toml:23`, `Cargo.lock:944-950` |
| Clipboard | arboard 3.4.1 (wayland-data-control) | `Cargo.toml:15`, `src/ui/talbe_view.rs:8,270` |
| CLI | clap 4 derive | `Cargo.toml:17`, `src/main.rs:22-36` |
| Errors | color-eyre (panic hook), anyhow | `src/main.rs:40,119-124`, `src/app.rs:95` |
| Web server | axum 0.7, tokio (full), tower-http CORS, rust-embed 8 | `Cargo.toml:16,27-28,22`, `src/api.rs:1-18` |
| Web front end | React 19 + Vite 6, built to `web/dist` and embedded | `web/package.json`, `src/api.rs:16-18` |

Other notes:
- AGENTS.md says rusqlite is "bundled by default" (`AGENTS.md:61`). That is wrong for this tree. `Cargo.toml:23` enables no `bundled` feature, and `Cargo.lock:944-950` lists libsqlite3-sys's deps as only `pkg-config` and `vcpkg`, with no `cc`. The binary therefore links a system SQLite (inferred from the lock file; not built to confirm).

## 3. Feature matrix

| Feature | Status | Evidence |
|---|---|---|
| Table list with row counts | Yes. Sidebar shows `name  count` | `src/ui/talbe_view.rs:294-318` |
| Views | Yes, Tables/Views toggle | `src/app.rs:352-381`, `src/ui/talbe_view.rs:1233-1236` |
| Schema view | Raw `sqlite_master.sql` text only, wrapped. No column types, keys or indexes | `src/ui/talbe_view.rs:475-485`; column names only, `src/app.rs:337-350` (takes PRAGMA column 1) |
| Data grid | Row-number column, cell cursor, horizontal scroll (20 cols), scrollbar, preview pane | `src/ui/talbe_view.rs:966-1050`, `679-741`, `288` |
| Paging | Fixed 500 rows per page, `LIMIT/OFFSET`; `u`/`d` page | `src/ui/talbe_view.rs:287`, `src/app.rs:233-256`, `1220-1231` |
| Sort / filter rows | None. Only the sidebar list is filterable | `src/ui/talbe_view.rs:367-380`, `1215-1219` |
| Query editor | Multi-line text, Ctrl+S runs all statements | `src/ui/talbe_view.rs:1119-1205`, `1274-1280` |
| Multi-statement split | Hand-written splitter respecting `'..'`, `".."`, `--`, `/* */` | `src/app.rs:12-84` |
| Per-statement results | `[` / `]` cycle results; errors red; DML shows "N rows affected" | `src/ui/talbe_view.rs:1188-1197`, draw_query at `743-965` |
| Autocomplete | Keywords + table/view names + `table.column`, by prefix of last word | `src/ui/talbe_view.rs:131-141`, `1282-1343` |
| SQL highlighting | Keyword list tokenizer for TUI editor | `src/ui/talbe_view.rs:143-228` |
| Query history | None (no "history" in src or web/src) | grep |
| Saved queries | None | grep |
| Copy to clipboard | Copy menu: cell, row, row+headers, all rows, CREATE SQL, query text | `src/ui/talbe_view.rs:522-635` |
| Export to file | None in TUI (no file writes in the tree). Web/API only: CSV, JSON, SQL INSERT | `src/api.rs:139-216`, `web/src/App.tsx:590-620` |
| Export page vs all (web) | "all" has no limit; server caps at 5000 rows for tables over 5000 (see section 8) | `web/src/App.tsx:597-598`, `src/api.rs:153-154` |
| DuckDB | None | grep |
| Read-only mode | None. All opens are read-write | `src/app.rs:190,217,242,263,288` (`Connection::open`) |
| Write support | Unrestricted. Any SQL is passed through (DDL, DML) | `src/app.rs:258-298` |
| File discovery | `./` only, non-recursive, `.db` `.sqlite3` `.db3` | `src/app.rs:138,398-423` |
| Help overlay | `?` toggles a key table | `src/ui.rs:56-58,71-73`, `src/ui/help_view.rs` |
| Web UI | Browse, Schema, Query, Export tabs (`L`, `H`, `;`, `X`) | `web/src/App.tsx:330`, `src/api.rs:44-52` |

## 4. Screens and keybindings

Screens: file menu (no DB) → table view with a left nav panel (tables/views list, search, hints) and a right panel (Browse | Schema | Query tabs). Help overlay on top of either. Layout is `src/ui.rs:45-59`, `src/ui/talbe_view.rs:382-520`.

Keys (from `src/ui/talbe_view.rs:1052-1272` and `src/ui.rs:61-75`, `src/main.rs:97-117`):

| Context | Key | Action |
|---|---|---|
| Global | `Ctrl+Q` | Quit (checked after dispatch, `src/main.rs:109-113`) |
| Global | `?` | Toggle help. Runs even when typing in the editor (see section 8) |
| File menu | `k`/`↑`, `j`/`↓`, `Enter` | Move, open DB (`src/ui.rs:77-95`) |
| Table view | `q` / `e` | Previous / next nav tab (Tables/Views) |
| Table view | `/` | Search the nav list; `Enter` applies, `Esc` clears |
| Table view | `H` / `L` / `;` | Schema / Browse / Query tab. `H`/`L` cycle, `;` jumps to Query |
| Table view | `i` / `k` (and `↑`/`↓`) | Cell up / down (row cursor) |
| Table view | `j` / `l` (and `←`/`→`) | Scroll left / right by 20 columns |
| Table view | `u` / `d` | Previous / next 500-row page |
| Table view | `y` | Yank current cell to clipboard |
| Table view | `c` | Copy menu |
| Table view | `Esc` | Back to Browse tab |
| Query tab | any char | Insert text (`Ctrl+...` not inserted) |
| Query tab | `Enter` | Newline, or accept suggestion if one is shown |
| Query tab | `Tab` | Accept suggestion |
| Query tab | `↑` / `↓` | Move through suggestions |
| Query tab | `Ctrl+S` | Run all statements |
| Query tab | `Ctrl+U` / `Ctrl+W` | Clear input / delete last word |
| Query tab | `[` / `]` | Previous / next result set |
| Query tab | `Esc` | Clear editor and results, back to Browse (no confirm) |
| Copy menu | `↑`/`↓`, `Enter`, `Esc` | Pick, copy, close |

Doc vs code mismatches (README and in-app hints):
- `README.md:99`, `src/ui/talbe_view.rs:453`, `src/ui/help_view.rs:37`: "Backspace: back to file menu". No Backspace handler exists outside search and query modes (`src/ui/talbe_view.rs:1097`, `1170`). Not implemented.
- `README.md:109`: "u / d: page up / down half". Code pages a full 500-row page (`src/ui/talbe_view.rs:1220-1231`).
- `README.md:96`: "q / e" is listed as switching Tables/Views. Matches code, but `q` is "previous" and `e` is "next".

## 5. Web UI and HTTP API (`--ui`)

- Started with `sqltui --ui -f db.sqlite3 --listen 5000` (`src/main.rs:43-53`). Server binds `0.0.0.0` (`src/api.rs:57`) with `CorsLayer::permissive()` (`src/api.rs:53`) and no auth.
- Routes (`src/api.rs:44-52`): `GET /api/info`, `/api/tables`, `/api/table/:name/data?offset&limit` (default limit 500, `:97`), `/api/table/:name/schema`, `/api/table/:name/export/:format` (csv|json|sql), `POST /api/query`, `POST /api/script` (multi-statement).
- `/api/query` and `/api/script` execute arbitrary SQL, including writes. Anyone on the network can use them.
- Body limit 10 MB (`src/api.rs:54`).
- The React app (`web/src/App.tsx`) uses page size 500 (`:168`), has a SQL editor with Ctrl+S and Tab/Enter suggestions (`:464-469`), and an Export tab (`:590-620`).

## 6. How it reads SQLite

- Driver: rusqlite, synchronous, with `Value`/`ValueRef` mapping (`src/app.rs:301-328`).
- Connection mode: a new `Connection::open(path)` for every call (`src/app.rs:190,217,242,263,288`). AGENTS.md calls this intentional (`AGENTS.md:38,70`). No WAL/journal settings, no `immutable`, no `query_only`, no read-only flags.
- Catalog: `SELECT type,name,sql,tbl_name FROM main.sqlite_master` (`src/app.rs:140-142,352-381`). Views are separate from tables.
- Columns: `PRAGMA table_info` (names only, `src/app.rs:337-350`).
- Row counts: `SELECT COUNT(*)` for every table and view at open, cached in `Table.row_count` (`src/app.rs:193-200,330-335`).
- Browse: if cached count > 500, `SELECT * FROM "t" LIMIT 500 OFFSET n` (`src/app.rs:240`, `src/ui/talbe_view.rs:1403-1427`). Otherwise `SELECT *` with no limit.
- Paging strategy: offset-based, no ORDER BY, no keyset. Row order is whatever SQLite returns.
- Value mapping: integer and real become strings; NULL becomes a `Null` marker; text that is not valid UTF-8 and all blobs become a `Blob` marker (`src/app.rs:301-328`). Type information is lost.
- Query results: fully materialized as `Vec<Vec<String>>`; no LIMIT is added to user SQL (`src/app.rs:107-129,270-273`).
- Blob cells in query results are shown as `<blob N bytes>` (`src/app.rs:117-119`).

## 7. Performance tricks (what they actually do)

1. Fixed 500-row page with LIMIT/OFFSET; small tables are loaded whole (`src/ui/talbe_view.rs:287,1423-1427`).
2. Row counts computed once at open and cached for sidebar and paging math (`src/app.rs:193-200`).
3. Ratatui stateful table draws only visible rows (`src/ui/talbe_view.rs:1048`). Column widths and Row objects are still built for all 500 loaded rows on each frame (`src/ui/talbe_view.rs:977-1019`).
4. Blobs are represented by a marker in the browse path (`src/app.rs:322`), so their contents are not shown or copied there.
5. Event loop blocks on `event::read()` and only redraws on input (`src/main.rs:102-108`). No timers, no background work.
6. Sidebar filter works on in-memory name lists (`src/ui/talbe_view.rs:367-380`).
7. Export endpoint caps at 5000 rows unless a limit is given (`src/api.rs:153-154`). Data endpoint defaults to 500 (`src/api.rs:97`).

None of these are async or cancellable. All DB calls run on the UI thread (TUI) or inside tokio handlers while holding a `tokio::sync::Mutex` over `App` (`src/api.rs:21-24,91,125,144,227,241`), with blocking rusqlite calls and no `spawn_blocking`.

## 8. Bugs and risks found by reading (not reproduced)

Ordered by impact.

1. Writes are unguarded and not reflected in the UI. SQL runs with no read-only mode and no confirmation (`src/app.rs:258-298`). `run_sql` never reloads `App.current_db` (only `load_db` callers are `src/ui.rs:84`, `src/main.rs:50,90`). So after DDL or DML the sidebar counts, the new tables and the paging totals are all stale. `load_table_data` also returns early when the same table is selected (`src/ui/talbe_view.rs:1409-1411`), so the grid is not refreshed on `Esc`.
2. Web API is open to the network. `0.0.0.0` bind, permissive CORS, unauthenticated `/api/query` and `/api/script` (`src/api.rs:50-57`).
3. `?` is handled globally after the tab handler (`src/ui.rs:61-73`). Typing `?` in the SQL editor both inserts the character and toggles help.
4. Copy-menu index bug. In the Query tab, "SQL query" is appended as index 5 (`src/ui/talbe_view.rs:533-535`). `do_copy` maps index 5 to the CREATE statement of the selected nav table (`:613-624`), so index 6 (the query text, `:626`) is unreachable from the menu. Needs a live check.
5. Query result grid cannot scroll. It is rendered with `render_widget`, not a stateful table (`src/ui/talbe_view.rs:937`), and no key moves its offset. Rows past the first screen are unreachable.
6. Naive statement splitter (`src/app.rs:12-84`). It does not understand `CREATE TRIGGER ... BEGIN ...; END;` (semicolons inside the body), backtick or `[...]` identifiers. Such input is split incorrectly.
7. `Esc` in the Query tab silently discards the editor text and all results (`src/ui/talbe_view.rs:1122-1131`).
8. Web "Export all" truncates silently. The client sends no limit (`web/src/App.tsx:597-598`), and the server caps tables over 5000 rows at 5000 (`src/api.rs:153-154`).
9. Unbounded memory: user queries and export output are fully materialized (`src/app.rs:107-129`, `src/api.rs:159-215`).
10. Synchronous DB work on the UI thread with no timeout or cancel (`src/main.rs:102-108`). A slow `COUNT(*)` at open or a long user query freezes the UI.
11. Open-time cost: `COUNT(*)` on every table and view (`src/app.rs:193-200`) is a full scan each. Expensive on large files.
12. `rows_affected` for SELECT in `/api/query` reports the row count (`src/api.rs:231`). Minor.
13. Documentation drift: `AGENTS.md:61` (bundled rusqlite) and `README.md:99,109` (see section 4).

## 9. Mapping onto opentui 0.9.9

Widgets available in `opentui::widget` (crate docs, docs.rs): View, Text, Input, Textarea, Select, TabSelect, Slider, ScrollBox, Table, TextTable, Code, LineNumbers, Diff, Markdown, TerminalView, ImageView, VideoView, FrameBuffer, Custom. App provides an event loop with timers and cross-thread messages.

Clean fits:
- Tables/Views switch: `TabSelect`.
- Browse/Schema/Query tabs: `TabSelect`.
- Nav list with counts and filter: `Select`, with the filter applied app-side (no list-search API seen in its method list).
- Schema view: `Code` (read-only, line numbers). Syntax highlight only if a grammar is present; the optional tree-sitter grammars listed on the crate page are javascript, md, rust, typescript and zig, with no SQL.
- SQL editor: `Textarea` (multi-line, undo, wrap, Alt+Enter submit). `Textarea` has extmarks (`add_extmark`, `extmarks`), which could carry keyword highlighting. Unverified.
- Autocomplete popup: `Select` inside an overlay `View`.
- Result grid: `Table` for row-level viewing. Its methods are `set_rows`, `push_row`, `row_count`, `selected`, `select`, `striped`, `on_select`. Suited to a 500-row page as a row vector.
- Preview and help: `Text` / `Markdown` in a `View`.
- Copy menu: `Select` in an overlay.
- Scroll containers: `ScrollBox`.

Needs custom work (`Custom` widget, or a composed grid):
- Per-cell cursor with row and column selection and column horizontal scroll. `Table` appears row-selected (`selected`/`select`), so sqltui's cell model is a custom widget (verify with a spike).
- Per-cell preview as in `draw_preview` with Blob/Null styling: compose `Text`.
- Per-statement results with a tab strip and error styling: compose `TabSelect` + `Table`/`Text`.
- Keyword highlight for SQL: depends on extmarks or a custom render.
- DB work off the UI thread: use opentui's messages-from-other-threads. Not needed by sqltui itself, which is synchronous.

## 10. Rust crate options (verified on crates.io, 2026-11-26)

| Crate | Latest | Rust (MSRV) | Crate size | Default features | Bundled feature |
|---|---|---|---|---|---|
| `rusqlite` | 0.43.0 | 1.89.0 | 184,812 B | `cache`, `ffi-sqlite-wasm-rs` | `bundled` = `libsqlite3-sys/bundled` + `modern_sqlite` |
| `libsqlite3-sys` | 0.40.0 | 1.88.0 | 2,759,453 B | `min_sqlite_version_3_45_3` | `bundled` = `cc` + `bundled_bindings` |
| `duckdb` | 1.20001.0 | 1.88 | 241,303 B | `libduckdb-sys/default` | `bundled` = `libduckdb-sys/bundled` |
| `libduckdb-sys` | 1.20001.0 | 1.88 | 9,216,953 B | `capi-v1`, `vcpkg`, `pkg-config`, `download-lib` | `bundled` = `cc` + `sha2` |
| `opentui` | 0.9.9 | 1.88 | 501,984 B | MIT | n/a |

Sources for the table: crates.io `/api/v1/crates/<name>/<version>` (feature maps, `crate_size`, `rust_version`); docs.rs for `OpenFlags::SQLITE_OPEN_READ_ONLY` (rusqlite) and `Config::access_mode` / `AccessMode::ReadOnly` / `Connection::open_with_flags` (duckdb). sqltui pins rusqlite 0.33 and libsqlite3-sys 0.31 (`Cargo.lock`).

Read-only support:
- rusqlite: `Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)`.
- duckdb: `Config::access_mode(AccessMode::ReadOnly)` with `Connection::open_with_flags`.

Cost and implications (what was checked and what was not):
- Checked: feature wiring, crate sizes (the crate source is what gets compiled), MSRVs.
- Not checked: compile time, resulting binary size, runtime behaviour. Nothing was built.
- rusqlite `bundled`: compiles SQLite C source via `cc`. Produces a statically linked SQLite, no system library needed, so single-binary distribution is simple. Expect a short added compile step; size impact expected to be small (unmeasured).
- duckdb default features do not bundle. `download-lib` (which pulls in `ureq`) suggests a prebuilt libduckdb is fetched at build time; behaviour not verified, but if so it implies network during build and a dynamically linked library. A static build needs `bundled` (C++ compile from the 9 MB crate source, requires a C++ toolchain, large compile time and memory). Unmeasured; a spike with `cargo build --timings` would settle it.
- `bundled` for duckdb also pulls extra features on demand: `json` and `parquet` imply `bundled`; `bundled-cmake` needs cmake and adds parquet; `bundled-icu` adds ICU data.
- MSRV: rusqlite 0.43 needs 1.89, one minor version above opentui and the DuckDB/SQLite sys crates (1.88). Pin the fmc toolchain accordingly.

## 11. Features worth porting (ideas only, no code copied)

Worth porting:
- Sidebar of tables and views with counts, plus a name filter.
- Schema tab showing the stored CREATE text, copyable.
- Paged grid with row numbers, cell preview, and a clear "rows x-y of N" title.
- Multi-statement query runner with per-statement result tabs and "N rows affected" for DML.
- Keyword and schema-name autocomplete with `table.column` completion.
- Copy cell / row / row-with-headers / all rows / CREATE / query text.
- Export CSV, JSON and SQL INSERT (fmc's version should write to a file, which sqltui does not do).

Skip, or do differently:
- Read-write by default with no guard. fmc should default to read-only and make writes explicit.
- Connection-per-call. Use one read-only connection per open file.
- Eager `COUNT(*)` for every table at open. Count lazily or on demand.
- Offset paging without ORDER BY. Use a stable order (rowid or key) when possible.
- Stringly typed cells. Keep the SQLite type for alignment, NULL styling and sorting.
- Unbounded query materialization. Cap rows or stream.
- Web server with an unauthenticated `0.0.0.0` bind and permissive CORS. Out of scope.
- Global `?` handling while editing, silent Esc discard, stale grid after writes, and the copy-menu index bug.
- Hand-rolled SQL splitter. Use a tokenizer-aware splitter or SQLite's own statement parsing (API not checked here).
- DuckDB is absent. Any DuckDB support is new design work, not a port.

## 12. Not done

- Nothing was compiled or run. Bugs in section 8 are from reading; the copy-menu bug, the stale-grid behaviour and the splitter failure should be confirmed in a live session.
- Binary size and compile times for rusqlite bundled and duckdb bundled were not measured.
- The `Textarea` extmark path for SQL highlighting and `Table`'s cell selection were inferred from the opentui method lists, not tested.
- The `web/dist` directory is listed in `.gitignore` but present in the clone; not investigated further.
