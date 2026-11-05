<img src="./images/mountains-logo.svg" width="128" alt="Mountains logo">

# Mountains

## For mindfulness and motivation on the trails

### Trail Running Training Log

#### _A digital tool to help runners **get good at vert** : )_

![Title on Startup](./images/mountains-tui-screenshot.png)

This is a TUI training log for trail running and food awareness.

Track daily nutrition, body measurements, miles, elevation gain, strength & mobility work, and notes.

A *sokay* (/soh-KAY/) is a moment you stray from your ideal way of eating: log it, forgive it, keep going. The idea comes from [Sokay](https://github.com/papadavis47/sokay).

Works offline-first with a local `libsql` database at `~/.mountains/`.

Optional Turso Cloud sync can be configured from within the app (press `c` on the startup screen).

Create a Turso account at [turso.tech](https://turso.tech/) and create a read / write libsql db. Copy over your database url and create a db auth token to input into the TUI.

`~/.mountains/` directory stores `config.toml` and `libsql` database files as well as markdown backup files of each day's entry.

# Installation

Clone the repo and install locally:

```shell

git clone git@github.com:papadavis47/mountains.git

cd mountains

cargo install --path . --locked

# binary builds to ~/.cargo/bin

```

The `~/.mountains/` directory is created automatically on first run.

# Cloud Sync (Optional)

Cloud sync with Turso is opt-in. Configure it from the startup screen (`c`) or edit `~/.mountains/config.toml` directly:

```toml
[sync]
enabled = true
db_url = "libsql://your-db.turso.io"
auth_token = "your-token"
```

Syncs on startup (background) and on quit.

# Usage

```shell
cargo run
```

Or after installing:

```shell

mountains

```

## Flags

| Flag | Description |
| --- | --- |
| `-t`, `--today` | Open today's log directly, skipping the startup screen |
| `-s`, `--simple` | Open today's log in a compact quick-entry view (miles, elevation, food, notes only) |
| `-n`, `--notes` | Open today's notes in a compact view |
| `-f`, `--food` | Open today's food and sokay in a compact view |
| `-e`, `--exercise` | Open today's strength & mobility in a compact view |
| `--resync-backups` | Clean up markdown backups: pull from Turso (if sync is on), write/update one `.md` per day in the DB, delete `.md` files for days no longer in the DB, then exit. Aborts without changes if Turso can't be reached |
| `-h`, `--help` | Print help |
| `-V`, `--version` | Print version |

Flags don't combine; only the first one is used. In the compact views, `Esc` goes to the log list and the rest of the session uses the full app.

With `cargo run`, pass flags after `--`, e.g. `cargo run -- -n`.

### Made with [ratatui](https://ratatui.rs/) :)
