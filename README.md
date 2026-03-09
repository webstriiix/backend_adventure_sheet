# Adventure Sheets

A **Rust** backend API for managing D&D 5e character sheets. Built with [Axum](https://github.com/tokio-rs/axum) and [SQLx](https://github.com/launchbait/sqlx) on top of PostgreSQL, it provides a RESTful API for character creation, advancement, inventory, spells, actions, and more.

## Features

- 🧙 **Character Management** — Create, update, and delete characters with full stat tracking
- ⚔️ **Inventory** — Add and manage weapons, armour, and equipment
- 📖 **Spells & Spell Slots** — Track prepared spells and slot expenditure
- 🎯 **Actions** — Class actions, Channel Divinity, Lay on Hands, and other features
- 🏆 **Feats & ASIs** — Ability Score Improvements and feat selection at level-up
- 🧪 **Compendium** — Browse races, classes, subclasses, spells, items, feats, backgrounds, and monsters
- 🔐 **Authentication** — JWT-based user auth with secure password hashing (Argon2)
- 🗄️ **Auto-migrations** — Database schema is applied automatically at startup

## Tech Stack

| Layer | Technology |
|-------|------------|
| Language | Rust (Edition 2024) |
| Web Framework | Axum 0.8 |
| Database | PostgreSQL (via SQLx 0.8) |
| Auth | JWT + Argon2 |
| Async Runtime | Tokio |
| Containerisation | Docker Compose |

## Prerequisites

- [Rust](https://rustup.rs/) (stable)
- [PostgreSQL](https://www.postgresql.org/) 14+ **or** [Docker](https://www.docker.com/)
- `sqlx-cli` (optional, for manual migrations)

## Getting Started

### 1. Clone the repository

```bash
git clone https://github.com/yourname/adventure_sheets.git
cd adventure_sheets
```

### 2. Configure environment variables

Copy the example file and fill in your values:

```bash
cp .env.example .env
```

| Variable | Description |
|----------|-------------|
| `DATABASE_URL` | PostgreSQL connection string e.g. `postgres://user:password@localhost/adventure_sheet_DB` |
| `JWT_SECRET` | Secret key used to sign JWT tokens |
| `PORT` | Port the server listens on (defaults to `8080`) |

### 3. Start the database

Using Docker Compose:

```bash
docker compose up -d
```

Or point `DATABASE_URL` at an existing PostgreSQL instance.

### 4. Run the server

```bash
cargo run
```

Migrations are applied automatically on startup. The API will be available at `http://localhost:8080`.

## API

All endpoints are prefixed with `/api/v1`.

📄 **Full endpoint reference, request/response schemas, and example payloads are documented in [API_DOCUMENTATION.md](./API_DOCUMENTATION.md).**

Key endpoint groups:

| Prefix | Description |
|--------|-------------|
| `/auth` | Register & login |
| `/characters` | CRUD for characters |
| `/characters/{id}/inventory` | Inventory management |
| `/characters/{id}/spells` | Spell tracking |
| `/characters/{id}/spell-slots` | Spell slot expenditure |
| `/characters/{id}/actions` | Class actions & features |
| `/characters/{id}/feats` | Feat management |
| `/compendium` | Browse game data |
| `/import` | Bulk import game data from JSON |

## Project Structure

```
src/
├── config.rs          # Environment config
├── db.rs              # DB pool & app state
├── error.rs           # Unified error types
├── main.rs            # Server entry point
├── handlers/          # Route handlers (auth, characters, compendium, …)
├── importers/         # JSON data importers
├── models/            # SQLx model structs
├── routes/            # Route definitions
└── services/          # Business logic
migrations/            # SQL migration files (auto-applied)
```

## Data Import

The server exposes a `POST /api/v1/import` endpoint that accepts JSON game data files (races, classes, spells, items, feats, etc.).

An annotated example of the expected JSON structure is provided in [`import_example.json`](./import_example.json). It covers the top-level keys the importer recognises:

```json
{
  "_meta": { "source": "example", "version": "0.0.1" },

  "race":       [ { "name": "...", "source": "...", "size": ["M"], "speed": 30, "ability": [...], "entries": [...] } ],
  "class":      [ { "name": "...", "source": "...", "hd": { "number": 1, "faces": 8 }, "proficiency": [...] } ],
  "subclass":   [ { "name": "...", "className": "...", "source": "...", "entries": [...] } ],
  "background": [ { "name": "...", "source": "...", "entries": [...] } ],
  "spell":      [ { "name": "...", "level": 1, "school": "A", "time": [...], "range": {...}, "duration": [...], "entries": [...] } ],
  "item":       [ { "name": "...", "type": "...", "rarity": "none", "entries": [...] } ]
}
```

To import a file, POST it directly to the endpoint:

```bash
curl -X POST http://localhost:8080/api/v1/import \
  -H "Content-Type: application/json" \
  -d @your_data_file.json
```

> **Important:** No game data is bundled with this repository. You must supply your own JSON files. See the [Disclaimer](#disclaimer) below.

## License

See [LICENSE](./LICENSE).

---

## Disclaimer

This project is an unofficial fan-made tool and is not affiliated with
Wizards of the Coast or D&D Beyond.

No copyrighted game content is included in this repository.
Users must provide their own JSON data files for races, classes,
spells, and other game data.
