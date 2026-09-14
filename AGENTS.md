# Mountains Training Log

A terminal-based training and nutrition tracking application built with Rust and ratatui.

Current release: **v0.9.1**.

## Project Overview - test

This is a TUI (Terminal User Interface) application for tracking daily training activities, nutrition, and body measurements with the following features:

- **Startup screen** - ASCII art logo with elevation statistics (monthly 1000+ days, yearly total, active streaks)
- **Branded title bars** - compact `▂▄▆█▆▄▂` Unicode logo on every non-startup title bar
- **Daily food logging** with date navigation
- **Body measurements** - weight and waist size tracking
- **Activity tracking** - miles covered (walking/hiking/running) and elevation gain with current ISO week, calendar month, and calendar year totals
- **Sokay tracking** - accountability for unhealthy food choices with cumulative counting
- **Strength & mobility tracking** - multi-line text field for logging exercises
- **Daily notes** for observations and reflections
- **Full CRUD operations** - add, edit, and delete entries for food and sokay items, plus delete entire days
- **Cursor-enabled text input** with arrow key navigation
- **Dual persistence** - libsql database (primary) with markdown file backups, optional Turso Cloud sync
- **Cloud sync** - syncs on startup and shutdown (with visual progress indicator), and pushes each save and day deletion in the background
- **Clean, responsive interface** with keyboard shortcuts

## Technology Stack

- **Rust** - Systems programming language
- **ratatui** - Terminal UI framework ([docs](https://docs.rs/ratatui/latest/ratatui/index.html))
- **crossterm** - Cross-platform terminal manipulation
- **chrono** - Date/time handling
- **serde** - Serialization for data persistence
- **libsql** - Embedded database with Turso Cloud sync
- **tokio** - Async runtime for database operations

## Key Controls

### Startup Screen

The startup screen appears when the application launches without flags (`-t`/`-s` bypass it). It displays:

- **ASCII art logo** - "MOUNTAINS" in large text art (centered)
- **Subtitle** - "For Inspiration and Mindfulness"
- **Monthly 1000+ days** - Count of days in current month with ≥1000 feet elevation
- **Yearly total** - Total feet of elevation gain for the current calendar year
- **Streak tracker** - Current consecutive days with ≥1000 feet elevation (minimum 2 days)

#### Startup Screen Controls:

- `n` - Go to Today's Log (creates if doesn't exist and opens DailyView)
- `l` - Go to Log List (opens Home screen with all daily logs)
- `a` - Add entry for a past date (opens date input modal, MM.DD.YYYY format)
- `s` - Open the Statistics screen
- `c` - Open Cloud Sync configuration (ConfigSync screen)
- `q` - Quit application (syncs with Turso Cloud if online)

### Statistics Screen

Accessed only from the Startup screen via lowercase `s`. It displays accumulated mileage and elevation for:

- **Current ISO week** - Monday through Sunday, identified by the complete ISO week-year and week number so weeks spanning New Year's Day are counted correctly
- **Current calendar month**
- **Current calendar year**

It also retains the monthly 1000+ foot day count and active elevation streak. `Esc` returns to Startup and `q` quits through the normal sync flow. Statistics use one captured reference date per render so every period shown is calculated against the same day.

### Mouse Input

Mouse input is additive; all keyboard controls remain available. A left-button click can activate visible Startup and Statistics footer actions, open a visible Home date, edit Daily View numeric/text sections, select Food/Sokay rows, and focus or toggle Cloud Sync fields. Clicking an already selected Food/Sokay row opens its editor. Mouse actions are derived from the regions actually rendered, including responsive footer tiers and list scroll offsets. Right clicks, mouse-up, movement, scrolling, hidden rows, modal backdrops, and unsupported screens do nothing.

### Home Screen

The home screen starts with the list **unfocused** (no item highlighted).

#### When list is unfocused:

- `Enter` - Go to today's log (creating if needed)
- `j` or `↓` - Focus list and select first item (most recent date)
- `k` or `↑` - Focus list and select last item (oldest date)

#### When list is focused:

- `↑/↓` or `j/k` - Navigate between dates
- `Enter` - Go to selected date's log
- `Esc` - Unfocus the list (remove highlight)
- `d` - Delete selected day (with confirmation)

#### Always available:

- `a` - Add entry for a past date (opens date input modal)
- `S` - Go to Startup Screen
- `q` - Quit application (syncs with Turso Cloud if online)

### Date Input Screen

- **Text input** accepting digits and dots only (MM.DD.YYYY format)
- `←/→` - Move cursor within text
- `Backspace/Delete` - Remove characters
- `Enter` - Parse date and navigate to DailyView (rejects future dates and invalid formats with red border error)
- `Esc` - Cancel and return to Home
- Error clears on next keystroke

### Daily View

The daily view shows all sections for tracking your training day: **Measurements**, **Running**, **Food Items**, **Sokay**, **Strength & Mobility**, and **Notes**.

#### Section Navigation

- `Shift+J` - Move focus to next section (down)
- `Shift+K` - Move focus to previous section (up)
- **Navigation order:** Measurements → Running → Food Items → Sokay → Strength & Mobility → Notes → (wraps to Measurements)
- The focused section has a **bright colored border** (yellow, red, cyan, magenta, or green depending on section)
- Unfocused sections have **dimmed gray borders**

#### Field Navigation (Measurements & Running sections only)

- `Tab` - Toggle between fields within a section
  - **Measurements:** Weight ↔ Waist
  - **Running:** Miles ↔ Elevation
  - While editing a numeric field, Tab first saves the value and then performs the same bidirectional toggle
- The focused field is indicated with a **► symbol**
- **Unset fields** show a dimmed inline placeholder (e.g. `Weight: Press 'w' to add`) in place of the value; it disappears per-field once that field has a value, mirroring the Food/Sokay/Notes placeholders

#### List Navigation (Food Items & Sokay sections only)

Lists start **unfocused** (no item highlighted) for quick access to adding new entries.

**When list is unfocused:**

- `j/↓` - Focus first item in the list
- `k/↑` - Focus last item in the list
- `Enter` - Add new entry (same as when focused)
- `e` and `d` - Do nothing (no item to edit/delete)
- `Esc` - Return to home screen

**When list is focused (item highlighted):**

- `↑/↓` or `j/k` - Navigate between items
- `e` - Edit selected item
- `d` - Delete selected item (with confirmation)
- `Esc` - Unfocus the item (remove highlight), next Esc returns to home

#### Expanded View (Strength & Mobility and Notes sections only)

When you navigate to the **Strength & Mobility** or **Notes** section, the section automatically expands if needed to show the full text content.

**Expansion behavior:**

- Expands **vertically upward** (bottom border stays fixed, top border moves up)
- **Overlays** sections above it (no other sections move or resize)
- Only expands if content doesn't fit in the default 4-line height
- Maximum expansion: **60% of screen height**
- If content exceeds 60%, the section becomes **scrollable**

**Scrolling:**

- `↑/k` - Scroll up (when section is expanded and has more content)
- `↓/j` - Scroll down (when section is expanded and has more content)
- Scrolling is **bounded to the content** — you cannot scroll past the last line into empty space (clamped via `max_scroll_offset`)
- Scroll position **resets to top** when navigating away with Shift+J/K
- Press `Enter` to edit (same as before)

**Use case:** Easily browse longer training notes and exercise logs without squinting at the small default section. Navigate to the section with Shift+J/K, read the full content, then press Enter if you want to edit.

#### Editing Data

- `Enter` - Edit the focused section/field or add new entry (for Food/Sokay)
  - **Measurements/Running:** Edits the focused field **in place** (no popup) — the value becomes a live input cell with a cursor right in the section row; the sibling field stays visible
  - **Food Items:** Opens "Add Food" dialog
  - **Sokay:** Opens "Add Sokay" dialog
  - **Strength & Mobility/Notes:** Opens editor for that section

##### Auto-advance on save

After saving a **single-value** field with Enter and a non-empty value, focus moves to the next field in entry order — no manual Shift+J needed. Order: Weight → Waist → Miles → Elevation → Food → Sokay → Strength & Mobility → Notes → (wraps) Weight.

For numeric fields, Tab is an alternate save action that preserves pair toggling instead of following the Enter sequence: Weight ↔ Waist and Miles ↔ Elevation. Tab saves empty values too, so clearing an existing value and pressing Tab clears it before toggling.

- **Focus-only, not edit-ready:** the next field is highlighted but not opened for typing; press Enter (or a quick-access key) to edit it, or Shift+K to go back. This applies uniformly to every single-value field, including the wrap from Notes back to Weight.
- **Empty Enter save stays put:** if the value is left blank, focus remains on the field (no advance); numeric Tab still toggles after saving an empty value.
- **Food/Sokay are exempt as sources:** their Add dialogs keep letting you enter item after item; you move on manually when done. Advancing *into* Food/Sokay (e.g. from Elevation) just focuses the section — no dialog auto-opens.
- Implemented via `SectionNavigator::advance_field` / `field_section` (`events/handlers.rs`), applied in the save branch of `handle_field_input` (`app/input.rs`).

#### Shortcuts Overlay

- `Space` - Toggle shortcuts help overlay (shows all quick access shortcuts)
  - Press Space again or `Esc` to close the overlay
  - Displays data entry shortcuts grouped by category (Measurements, Activity, Nutrition, Training)
  - Only works when not typing in an input field

#### Quick Access Shortcuts

These shortcuts allow quick data entry without navigating sections. Press **Space** to see the full list in an overlay.

- `w` - Edit weight measurement
- `s` - Edit waist measurement
- `m` - Edit miles covered
- `l` - Edit elevation gain
- `f` - Add new food item
- `c` - Add new sokay entry
- `t` - Edit strength & mobility exercises
- `n` - Edit daily notes
- `S` - Go to Startup Screen
- `Esc` - Back to home screen

### Add/Edit Food Screens

- **Text input** with full cursor support
- `←/→` - Move cursor within text
- `Home/End` - Jump to beginning/end
- `Backspace/Delete` - Remove characters
- `Enter` - Save entry
- `Esc` - Cancel and return

### Edit Measurements (Weight/Waist/Miles/Elevation) - In-Place

Weight, Waist, Miles, and Elevation are edited **in place** within their Measurements/Running section row — no popup modal. The value being edited becomes a live input cell with a blinking cursor; the other field in the row stays visible for context. The section's bright border and `►` marker indicate the active field.

- **Numeric input** (weight, waist, miles: decimal; elevation: integer only)
- `←/→` - Move cursor within text
- `Home/End` - Jump to beginning/end
- `Backspace/Delete` - Remove characters
- `Enter` - Save measurement and focus the next field in entry order
- `Tab` - Save measurement and toggle to the paired numeric field (Weight ↔ Waist or Miles ↔ Elevation)
- `Esc` - Cancel and return (restores original value)

### Edit Strength & Mobility Screen

- **Multi-line text input** with cursor support and newline insertion
- `←/→/↑/↓` - Move cursor
- `Home/End` - Jump to beginning/end of entire text
- `Alt+Enter` - Insert newline (hard line break, max 200 lines)
- `Enter` - Save exercises
- `Esc` - Cancel and return

### Edit Notes Screen

- **Multi-line text input** with cursor support and newline insertion
- `←/→/↑/↓` - Move cursor
- `Home/End` - Jump to beginning/end of entire text
- `Alt+Enter` - Insert newline (hard line break, max 200 lines)
- `Enter` - Save notes
- `Esc` - Cancel and return

### Delete Day Confirmation Screen

- `y` - Confirm deletion
- `n` or `Esc` - Cancel deletion and return to home screen

### Cloud Sync Config Screen

Accessed from Startup screen via `c`. Centered modal overlay (~60% width, ~50% height).

- **Database URL** - text input for Turso database URL
- **Auth Token** - masked text input (shows `****` if saved token exists)
- **Cloud Sync toggle** - Enabled/Disabled
- `Tab` - Cycle between fields (DbUrl → AuthToken → EnableToggle)
- `Space` on EnableToggle - toggles enabled/disabled
- `Enter` - saves config to `~/.mountains/config.toml` and returns to Startup (from any field)
- `Esc` - Cancel and return to Startup
- Status message shows "Saved!" in green on success
- If newly configured, spawns background cloud connection immediately

### Syncing Screen

The syncing screen appears automatically when quitting the application (pressing `q`).

- **Online mode:** Displays a centered modal with "Syncing with Turso Cloud..." message and a progress gauge
- **Offline mode:** Shows "Offline - changes will sync when network is available" message
- The screen automatically closes after sync completes (or immediately if offline)
- Uses a visual Gauge widget to show sync progress
- Border color changes based on status:
  - **Cyan** - Syncing in progress
  - **Green** - Sync complete
  - **Orange** - Offline mode

## Command-Line Flags

Parsed in `main.rs` (`handle_cli_args`) before the TUI starts, so they print to the normal terminal and never enter the alternate screen:

- `-t` / `--today` - Launch directly into today's DailyView (creating the log if needed), bypassing the Startup screen
- `-s` / `--simple` - Launch directly into today's log in **simple quick-entry mode** (see below); implies `-t`
- `-V` / `--version` - Print `mountains <version>` (version sourced from `CARGO_PKG_VERSION`, auto-synced to Cargo.toml) and exit 0
- `-h` / `--help` - Print usage/help (the `HELP_TEXT` const) and exit 0
- Unrecognized argument - Print error + help to stderr and exit 2
- No arguments - Launch the interactive TUI as normal

Flags don't combine; only the first argument is read. Parsing lives in `parse_cli_arg` (pure, tested) and `handle_cli_args`; the chosen `LaunchMode` is applied via `AppState::apply_launch_mode` after logs load.

### Simple Quick-Entry Mode (`-s`)

A compact DailyView variant for fast daily entry:

- Renders as a **centered window** (~70% width, green rounded border, interior padding of 3 cols / 1 row, title "Quick Entry - <date>") instead of the full-screen layout
- Shows only **Running (Miles, Elevation), Food Items, and Notes**; Measurements, Sokay, and Strength & Mobility are hidden
- Section order / auto-advance: Miles → Elevation → Food → Notes → (wraps) Miles
- Quick keys `m`, `l`, `f`, `n`, Space overlay, Tab pair-toggling, and all Food/Notes editing work as in full mode; `w`, `s`, `t`, `c`, and `S` are disabled (`SectionNavigator::daily_view_key_enabled`)
- Space overlay and footer help tiers list only the available shortcuts
- Mouse targets derive from what's drawn, so hidden sections aren't clickable
- `q` quits with the normal sync flow; `Esc` goes to Home **and ends simple mode** — the rest of the session behaves like the full app (re-opening a day shows the full view)

## File Structure

```
src/
├── main.rs              # Application entry point + CLI flag handling (--version/--help)
├── app/                 # Main App struct, split across an impl in sibling submodules
│   ├── mod.rs           # App struct + lifecycle (new/run/event dispatch/sync helpers)
│   ├── render.rs        # `ui`: per-screen render dispatch
│   ├── click.rs         # Mouse hit-testing → ClickAction handling
│   ├── input.rs         # Text/modal input handlers (food, sokay, fields, date, config, delete)
│   └── navigation.rs    # Key-driven navigation, selection, quick-access edits
├── config.rs            # AppConfig/SyncConfig with TOML persistence + .env migration
├── models.rs            # Data structures (FoodEntry, DailyLog, AppState, AppScreen)
├── assets.rs            # ASCII art and UI constants
├── period.rs            # Period enum (Week/Month/Year) + membership predicate, shared by stats
├── elevation_stats.rs   # Elevation statistics calculations
├── miles_stats.rs       # Miles statistics calculations
├── db_manager.rs        # Database operations with Turso Cloud sync
├── file_manager.rs      # Markdown serialization for write-only backups (+ delete)
├── events/
│   └── handlers.rs      # Event handlers (InputHandler, ActionHandler)
└── ui/
    ├── mod.rs           # UI module
    ├── components.rs    # Reusable UI components
    └── screens/
        ├── config_sync.rs # Cloud sync configuration modal
        ├── statistics.rs  # Weekly/monthly/yearly statistics screen
        └── ...            # Other screen modules

Data storage:
- Database: ~/.mountains/mountains.db (local libsql database)
- Config: ~/.mountains/config.toml (cloud sync credentials, TOML format)
- Backups: ~/.mountains/mtslog-MM.DD.YYYY.md (markdown files)
- Cloud: Synced to Turso Cloud on startup and shutdown (opt-in via config)
```

### Example Data File Format:

```markdown
# Mountains Training Log - January 09, 2025

## Measurements

- **Weight:** 175.5 lbs
- **Waist:** 34.2 inches

## Food

- Oatmeal
- Chicken Salad
- Green Tea

## Running

- **Miles:** 3.2 mi
- **Elevation:** 450 ft

## Sokay

- Coca Cola
- Chocolate bar

## Strength & Mobility

Pull-ups: 3x8
Push-ups: 3x15
Hip mobility stretches: 10 minutes

## Notes

Feeling strong today. Good hike in the morning.
```

## Development Commands

- `cargo run` - Run the application
- `cargo check` - Check for compilation errors
- `cargo build --release` - Build optimized binary
- `cargo fmt` - Format all code
- `cargo clippy --all-targets` - Lint

## Formatting & Pre-Commit Hook

This repo is kept **fully rustfmt-clean and clippy-clean (zero warnings)**. A
committed pre-commit hook enforces this on every commit.

- **Hook:** `.githooks/pre-commit` runs `cargo fmt --check` then
  `cargo clippy --all-targets -- -D warnings`. If either fails, the commit is
  blocked. To fix: run `cargo fmt`, resolve any clippy warnings, re-stage, commit.
- **One-time setup (per clone):** `git config core.hooksPath .githooks`. The hook
  file ships with the repo, but git won't use a custom hooks path until pointed at
  it; there is no way to auto-enable this from inside the repo. Verify with
  `git config --get core.hooksPath` (should print `.githooks`).
- **Escape hatch:** `git commit --no-verify` skips the hook (use deliberately).
- The hook checks the whole working tree, not just staged files, so unstaged drift
  in any file also blocks the commit until resolved.
- **Agents:** before committing Rust changes, ensure `cargo fmt` and
  `cargo clippy --all-targets` are clean; do not commit code that fails the hook.

## Useful Links

- [ratatui Documentation](https://docs.rs/ratatui/latest/ratatui/index.html)
- [ratatui Examples](https://github.com/ratatui-org/ratatui/tree/main/examples)
- [crossterm Documentation](https://docs.rs/crossterm/latest/crossterm/)

## Recent Improvements

### Latest Session (Snappy Input After Saving)

- ✅ **No stall after saving** - Saving a Food/Sokay entry no longer freezes `j`/`k` for the duration of the Turso sync; the network call runs with the `DbManager` write lock released
- ✅ **Sync status off the hot path** - `App` holds the `connection_state` Arc directly, so the event loop's per-frame status refresh never takes the `DbManager` lock
- ✅ **Dead code removed** - private `DbManager::sync` and `get_connection_state` dropped
- ✅ **Regression test** - a held write lock must not delay `update_sync_status` (fails by deadlock before the fix)

### Previous Session (Text-Hugging List Highlight)

- ✅ **Selection hugs the text** - Home, Food, and Sokay rows highlight only the row text plus one trailing space instead of the full section width
- ✅ **Shared helper** - `selectable_list_item` in `ui/components.rs` replaces the three `highlight_style` call sites
- ✅ **Test coverage** - `TestBackend` buffer assertions on the reversed-cell run for Home and Food, plus an unfocused-list no-highlight case

### Earlier Session (CLI Launch Flags and Simple Mode)

- ✅ **`-t`/`--today` flag** - Launches straight into today's DailyView, skipping Startup
- ✅ **`-s`/`--simple` flag** - Quick-entry mode: centered window with only Miles, Elevation, Food, and Notes
- ✅ **Testable arg parsing** - `handle_cli_args` refactored around pure `parse_cli_arg` → `CliAction`/`LaunchMode`
- ✅ **Mode-aware navigation** - `SectionNavigator` traversal/auto-advance skips hidden sections; Notes wraps to Miles
- ✅ **Key gating** - `w`/`s`/`t`/`c`/`S` disabled in simple mode; Space overlay and footer tiers show only live shortcuts
- ✅ **Exit semantics** - `Esc` → Home ends simple mode (full app afterwards); `q` quits with normal sync
- ✅ **Test coverage** - Parser, launch-mode application, simple traversal/advance, key gating, renderer hiding/centering, overlay content

### Previous Session (Title Branding and Numeric Tab Save)

- ✅ **Compact title logo** - Every non-startup title bar prefixes its title with the green, bold `▂▄▆█▆▄▂` mark
- ✅ **Tab saves numeric input** - Weight, waist, miles, and elevation accept Tab as a save action with bidirectional pair toggling
- ✅ **Enter behavior preserved** - Enter continues advancing through the existing top-to-bottom entry order

## Architecture Notes

- **App struct** - Main application coordinator managing state, database, and UI
- **State management** - AppScreen enum for view routing (screens including Startup, Statistics, DateInput, ConfigSync, and Syncing)
- **Dual persistence** - libsql database (primary) + markdown files (backup)
- **Offline-first design** - Local database initializes instantly, cloud connection deferred to background
- **Cloud sync** - Syncs on startup (background pull) and shutdown (with visual feedback), plus a detached push after every save and day deletion; graceful offline handling. Enabling sync on an existing local install is lossless: libsql can't convert a local db to a replica in-place, so `upgrade_to_remote_replica` (`db_manager.rs`) stashes the local db files aside (`mountains.db.pre-sync.<unix-ts>`, unique per attempt so a failed try never clobbers an earlier stash) instead of deleting them; after the replica's first **successful** pull, `import_stashed_dbs` inserts only dates the replica doesn't already have (remote wins per date), then removes the stash. If the pull or import fails, stashes stay on disk and are retried on next connect. The startup pull runs inside the detached connection task, and because the event loop no longer reads sync status through the `DbManager` lock (see **Lock discipline on the render loop**), that task's write lock cannot delay first paint; when it completes it sets an `Arc<AtomicBool>` (`needs_reload`) that the event loop drains via `reload_logs_if_needed`, reloading the `daily_logs` cache in place so rows written by other clients (e.g. a companion web app on the same Turso DB) appear a beat after launch
- **Connection state tracking** - Real-time monitoring of Turso Cloud connection status
- **Async architecture** - Fully async event loop and database operations using tokio
- **Thread-safe database** - Arc<RwLock<DbManager>> for shared access across async tasks
- **Input handling** - Specialized handlers for text, numeric, integer, and multi-line input
- **Activity statistics** - Dedicated elevation and mileage modules calculate ISO-week, calendar-month, and calendar-year totals from an explicit reference date
- **Modular design** - Separated concerns (models, events, ui, database, file management, stats)
- **Responsive UI** - Terminal size adaptation with ratatui layout system, live sync status display
- **Responsive footer help** - `render_help` (`ui/components.rs`) takes `tiers: &[&str]` ordered fullest→minimal and renders the widest tier that fits the (border-adjusted) area width, so the bottom help bar never truncates on narrow/split terminals; the last tier is the guaranteed fallback. Callers (daily_view, home, startup, confirmations) supply their own tiers. The full reference remains one keystroke away via the Space shortcuts overlay.
- **Text-hugging list selection** - The Home date list and the Daily View Food/Sokay lists draw selection with `selectable_list_item` (`ui/components.rs`), which styles the selected row's own span instead of setting `List::highlight_style`. Ratatui applies `highlight_style` (and `ListItem::style`) to the whole row rect, which stretched the reversed bar across the full section on a wide terminal; the span-based highlight covers the text plus one trailing space.
- **Lock discipline on the render loop** - The event loop must never wait on the `Arc<RwLock<DbManager>>`. Cloud sync is a network round-trip, so it runs with the write lock released: `save_daily_log`/`delete_daily_log` commit the local transaction only, and the caller (`persist_daily_log` in `events/handlers.rs`, `App::spawn_sync` for day deletion) pushes to Turso afterwards under a read lock. `App` also holds its own clone of the `connection_state` handle (`DbManager::connection_state_handle`), so `update_sync_status` reads sync status every frame without touching the `DbManager` lock at all. Regression test: `sync_status_refreshes_while_a_persist_holds_the_db_write_lock` (`app/mod.rs`).
- **Render-derived mouse targets** - Renderers optionally register `ClickTarget` regions for the controls and rows that were actually drawn. `App` clears and rebuilds this map every frame, reverse-order hit testing gives overlays priority, and modal backdrops render without targets to prevent click-through.
- **Data integrity** - Database transactions for atomic operations
- **Error handling** - anyhow for ergonomic error propagation

### Key Data Structures

- **DailyLog** - Main data model with food_entries, measurements, sokay_entries, strength_mobility, notes
- **AppState** - Application state with daily_logs cache, current screen/selection, focused_section, simple_mode, and date_input_error
- **LaunchMode** - CLI-chosen start screen (Startup/Today/Simple), applied by `AppState::apply_launch_mode`
- **FocusedSection** - Enum tracking which section has focus (Measurements, Running, FoodItems, Sokay, StrengthMobility, Notes)
- **MeasurementField** - Enum for tracking focus within Measurements section (Weight, Waist)
- **RunningField** - Enum for tracking focus within Running section (Miles, Elevation)
- **SectionNavigator** - Pure function-based navigation logic for section and field traversal
- **InputHandler** - Cursor position tracking and input validation
- **DbManager** - Async database operations with deferred cloud connection and state tracking
- **ConnectionState** - Enum tracking sync status (Disconnected, Connected, Error)
- **AppConfig/SyncConfig** - TOML-persisted config for cloud sync credentials (enabled, db_url, auth_token)
- **ConfigSyncField** - Enum tracking focus within ConfigSync screen (DbUrl, AuthToken, EnableToggle)
- **FileManager** - Markdown serialization for write-only backups (no deserializer; recovery/sync happens via the libsql database, not by reading the `.md` files)

# Rust coding guidelines

- Prioritize code correctness and clarity. Speed and efficiency are secondary priorities unless otherwise specified.
- Do not write comments that summarize the code. Comments should only be written in order to explain "why" the code is written in some way in the case there is a reason that is tricky / non-obvious. But do write comments that document APIs.

# important-instruction-reminders

Do what has been asked; nothing more, nothing less.
NEVER create files unless they're absolutely necessary for achieving your goal.
ALWAYS prefer editing an existing file to creating a new one.
NEVER proactively create documentation files (\*.md) or README files. Only create documentation files if explicitly requested by the User.

- always update @CLAUDE.md with changes
- clean up dead code as the app evolves and changes are made
