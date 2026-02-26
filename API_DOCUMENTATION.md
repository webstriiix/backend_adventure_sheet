# Adventure Sheets API Documentation

**Base URL:** `http://localhost:8080/api/v1`

## Authentication

All protected endpoints require a JWT Bearer token in the `Authorization` header:

```
Authorization: Bearer <token>
```

Tokens are issued on signup/login and expire after **30 days**.

### Error Responses

All errors return JSON:

```json
{
  "error": "Error message here"
}
```

| Status Code | Meaning |
|---|---|
| `400` | Bad Request |
| `401` | Unauthorized (missing/invalid token) |
| `404` | Not Found |
| `500` | Internal Server Error |

---

## Health Check

### `GET /check_health`

Simple liveness probe.

**Auth:** None

**Response:** `200 OK`
```
Server Good
```

---

## Auth

### `POST /signup`

Create a new user account.

**Auth:** None

**Request Body:**
```json
{
  "username": "guan_yu",
  "email": "guanyu@shu.com",
  "password": "five_tiger_general"
}
```

| Field | Type | Required |
|---|---|---|
| `username` | string | Yes |
| `email` | string | Yes |
| `password` | string | Yes |

**Response:** `201 Created`
```json
{
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "user": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "username": "guan_yu",
    "email": "guanyu@shu.com",
    "created_at": "2026-02-20T10:00:00Z"
  }
}
```

**Errors:**
- `400` — User already exists (duplicate username or email)

---

### `POST /login`

Authenticate an existing user.

**Auth:** None

**Request Body:**
```json
{
  "email": "guanyu@shu.com",
  "password": "five_tiger_general"
}
```

| Field | Type | Required |
|---|---|---|
| `email` | string | Yes |
| `password` | string | Yes |

**Response:** `200 OK`
```json
{
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "user": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "username": "guan_yu",
    "email": "guanyu@shu.com",
    "created_at": "2026-02-20T10:00:00Z"
  }
}
```

**Errors:**
- `401` — Invalid email or password

---

## Compendium (Public)

All compendium endpoints are public (no auth required) and read-only.

### `GET /classes`

List all classes.

**Query Parameters:**

| Param | Type | Required | Description |
|---|---|---|---|
| `source` | string | No | Filter by source slug (e.g. `PHB`) |
| `edition` | string | No | Filter by edition (e.g. `classic`) |

**Response:** `200 OK`
```json
[
  {
    "id": 1,
    "name": "Paladin",
    "source_slug": "PHB",
    "hit_die": 10,
    "proficiency_saves": ["wis", "cha"],
    "spellcasting_ability": "cha",
    "caster_progression": "1/2",
    "weapon_proficiencies": ["simple", "martial"],
    "armor_proficiencies": ["light", "medium", "heavy", "shield"],
    "skill_choices": {},
    "starting_equipment": {},
    "multiclass_requirements": {},
    "class_table": [],
    "subclass_title": "Sacred Oath",
    "edition": null
  }
]
```

---

### `GET /classes/{name}/{source}`

Get full class detail including all features and subclasses.

**Path Parameters:**

| Param | Type | Description |
|---|---|---|
| `name` | string | Class name (e.g. `Paladin`) |
| `source` | string | Source slug (e.g. `PHB`) |

**Example:** `GET /classes/Paladin/PHB`

**Response:** `200 OK`
```json
{
  "class": {
    "id": 1,
    "name": "Paladin",
    "source_slug": "PHB",
    "hit_die": 10,
    "proficiency_saves": ["wis", "cha"],
    "spellcasting_ability": "cha",
    "caster_progression": "1/2",
    "weapon_proficiencies": [],
    "armor_proficiencies": [],
    "skill_choices": {},
    "starting_equipment": {},
    "multiclass_requirements": null,
    "class_table": [],
    "subclass_title": "Sacred Oath",
    "edition": null
  },
  "features": [
    {
      "id": 1,
      "name": "Divine Sense",
      "source_slug": "PHB",
      "class_name": "Paladin",
      "level": 1,
      "entries": [],
      "is_subclass_gate": false
    }
  ],
  "subclasses": [
    {
      "subclass": {
        "id": 1,
        "name": "Oath of Devotion",
        "short_name": "Devotion",
        "source_slug": "PHB",
        "class_name": "Paladin",
        "class_source": "PHB",
        "unlock_level": 3,
        "fluff_text": null,
        "fluff_image_url": null
      },
      "features": [
        {
          "id": 1,
          "name": "Sacred Weapon",
          "source_slug": "PHB",
          "subclass_short_name": "Devotion",
          "subclass_source": "PHB",
          "class_name": "Paladin",
          "level": 3,
          "header": null,
          "entries": []
        }
      ]
    }
  ]
}
```

**Errors:**
- `404` — Class not found

---

### `GET /spells`

Search spells. **Limit: 100 results.**

**Query Parameters:**

| Param | Type | Required | Description |
|---|---|---|---|
| `name` | string | No | Substring search (case-insensitive) |
| `source` | string | No | Filter by source slug |

**Example:** `GET /spells?name=smite&source=PHB`

**Response:** `200 OK`
```json
[
  {
    "id": 1,
    "name": "Divine Smite",
    "source_id": 1,
    "level": 1,
    "school": "V",
    "casting_time": [{"number": 1, "unit": "bonus"}],
    "range": {"type": "point", "distance": {"type": "self"}},
    "components": {"v": true},
    "duration": [{"type": "instant"}],
    "entries": [],
    "entries_higher_lvl": null,
    "ritual": false,
    "concentration": false
  }
]
```

---

### `GET /items`

Search items. **Limit: 100 results.**

**Query Parameters:**

| Param | Type | Required | Description |
|---|---|---|---|
| `name` | string | No | Substring search (case-insensitive) |
| `source` | string | No | Filter by source slug |

**Example:** `GET /items?name=longsword`

**Response:** `200 OK`
```json
[
  {
    "id": 1,
    "name": "Longsword",
    "source_id": 1,
    "type": "M",
    "rarity": "none",
    "weight": "3",
    "value_cp": 1500,
    "damage": {"dmg1": "1d8", "dmgType": "S"},
    "armor_class": null,
    "properties": ["V"],
    "requires_attune": false,
    "entries": null,
    "is_magic": false
  }
]
```

---

### `GET /monsters`

Search monsters. **Limit: 50 results.**

**Query Parameters:**

| Param | Type | Required | Description |
|---|---|---|---|
| `name` | string | No | Substring search (case-insensitive) |
| `source` | string | No | Filter by source slug |

**Response:** `200 OK`
```json
[
  {
    "id": 1,
    "name": "Goblin",
    "source_id": 1,
    "size": ["S"],
    "type": "humanoid",
    "alignment": ["N", "E"],
    "ac": [{"ac": 15, "from": ["leather armor", "shield"]}],
    "hp_average": 7,
    "hp_formula": "2d6",
    "speed": {"walk": 30},
    "str": 8, "dex": 14, "con": 10,
    "int": 10, "wis": 8, "cha": 8,
    "skills": {"stealth": 6},
    "senses": ["darkvision 60 ft."],
    "passive": 9,
    "cr": "1/4",
    "traits": [],
    "actions": [],
    "reactions": null
  }
]
```

---

### `GET /races`

List races. **No result limit.**

**Query Parameters:**

| Param | Type | Required | Description |
|---|---|---|---|
| `name` | string | No | Substring search (case-insensitive) |
| `source` | string | No | Filter by source slug |

**Response:** `200 OK`
```json
[
  {
    "id": 1,
    "name": "Human",
    "source_id": 1,
    "size": ["M"],
    "speed": {"walk": 30},
    "ability_bonuses": [{"str": 1, "dex": 1, "con": 1, "int": 1, "wis": 1, "cha": 1}],
    "age_description": "Humans reach adulthood in their late teens...",
    "alignment_description": "Humans tend toward no particular alignment.",
    "skill_proficiencies": null,
    "language_proficiencies": [{"common": true, "anyStandard": 1}],
    "trait_tags": [],
    "entries": []
  }
]
```

---

### `GET /backgrounds`

List backgrounds. **No result limit.**

**Query Parameters:**

| Param | Type | Required | Description |
|---|---|---|---|
| `name` | string | No | Substring search (case-insensitive) |
| `source` | string | No | Filter by source slug |

**Response:** `200 OK`
```json
[
  {
    "id": 1,
    "name": "Soldier",
    "source_id": 1,
    "skill_proficiencies": [{"athletics": true, "intimidation": true}],
    "tool_proficiencies": [{"gaming set": true, "vehicles (land)": true}],
    "language_count": 0,
    "starting_equipment": {},
    "entries": []
  }
]
```

---

### `GET /optional-features`

List optional features (Fighting Styles, Eldritch Invocations, Metamagic, etc.). **Limit: 100 results.**

**Query Parameters:**

| Param | Type | Required | Description |
|---|---|---|---|
| `name` | string | No | Substring search (case-insensitive) |
| `source` | string | No | Filter by source slug |
| `feature_type` | string | No | Exact match on type code |

**Common `feature_type` values:**

| Code | Meaning |
|---|---|
| `EI` | Eldritch Invocation |
| `FS:F` | Fighting Style: Fighter |
| `FS:R` | Fighting Style: Ranger |
| `FS:P` | Fighting Style: Paladin |
| `MM` | Metamagic |
| `AI` | Artificer Infusion |
| `MV:B` | Maneuver: Battle Master |

**Example:** `GET /optional-features?feature_type=FS:P`

**Response:** `200 OK`
```json
[
  {
    "id": 1,
    "name": "Defense",
    "source_id": 1,
    "feature_type": "FS:P",
    "prerequisite": null,
    "entries": ["+1 bonus to AC while wearing armor."]
  }
]
```

---

## Characters (Auth Required)

All character endpoints require `Authorization: Bearer <token>`.
All operations are scoped to the authenticated user's characters.

### `POST /characters`

Create a new character.

**Request Body:**
```json
{
  "name": "Guan Yu",
  "class_id": 1,
  "race_id": 1,
  "subrace_id": null,
  "background_id": 1,
  "str": 16,
  "dex": 10,
  "con": 14,
  "int": 8,
  "wis": 12,
  "cha": 15,
  "max_hp": 12
}
```

| Field | Type | Required | Description |
|---|---|---|---|
| `name` | string | Yes | Character name |
| `class_id` | integer | Yes | Starting class (level 1, primary) |
| `race_id` | integer | No | Race FK |
| `subrace_id` | integer | No | Subrace FK |
| `background_id` | integer | No | Background FK |
| `str` | integer | Yes | Strength score |
| `dex` | integer | Yes | Dexterity score |
| `con` | integer | Yes | Constitution score |
| `int` | integer | Yes | Intelligence score |
| `wis` | integer | Yes | Wisdom score |
| `cha` | integer | Yes | Charisma score |
| `max_hp` | integer | Yes | Maximum hit points (`current_hp` set equal) |

**Response:** `200 OK`
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "user_id": "660e8400-e29b-41d4-a716-446655440000",
  "name": "Guan Yu",
  "experience_pts": 0,
  "race_id": 1,
  "subrace_id": null,
  "background_id": 1,
  "str": 16, "dex": 10, "con": 14,
  "int": 8, "wis": 12, "cha": 15,
  "max_hp": 12,
  "current_hp": 12,
  "temp_hp": 0,
  "inspiration": false,
  "notes": null,
  "created_at": "2026-02-20T10:00:00Z",
  "updated_at": "2026-02-20T10:00:00Z"
}
```

**Notes:** Also inserts into `character_classes` (class_id at level 1, is_primary=true) within a transaction.

---

### `GET /characters`

List all characters for the authenticated user.

**Response:** `200 OK` — Array of character objects (sorted by `updated_at DESC`).

---

### `GET /characters/{id}`

Get a single character by UUID.

**Path Parameters:**

| Param | Type | Description |
|---|---|---|
| `id` | UUID | Character ID |

**Response:** `200 OK` — Single character object.

**Errors:**
- `404` — Character not found or not owned by user

---

### `PUT /characters/{id}`

Update a character.

**Request Body:** Same shape as create. (`class_id` is accepted but **not used** in the update query.)

**Response:** `200 OK` — Updated character object.

**Errors:**
- `404` — Character not found or access denied

---

### `DELETE /characters/{id}`

Delete a character and all related data (feats, spells, inventory cascade).

**Response:** `204 No Content`

**Errors:**
- `404` — Character not found or access denied

---

## Character Feats (Auth Required)

### `GET /characters/{id}/feats`

List all feats for a character.

**Response:** `200 OK`
```json
[
  {
    "id": 1,
    "character_id": "550e8400-...",
    "feat_id": 5,
    "chosen_ability": "cha",
    "uses_remaining": 3,
    "uses_max": 3,
    "recharge_on": "long_rest",
    "source_type": "level",
    "gained_at_level": 4
  }
]
```

---

### `POST /characters/{id}/feats`

Add a feat to a character. Automatically resolves `uses_max` and `recharge_on` from the feat definition.

**Request Body:**
```json
{
  "feat_id": 5,
  "chosen_ability": "cha",
  "source_type": "level",
  "gained_at_level": 4
}
```

| Field | Type | Required | Default | Description |
|---|---|---|---|---|
| `feat_id` | integer | Yes | — | FK to feats table |
| `chosen_ability` | string | No | null | Which ASI the player chose |
| `source_type` | string | No | `"level"` | `"level"`, `"background"`, `"species"`, `"bonus"` |
| `gained_at_level` | integer | No | null | Character level when feat was taken |

**Response:** `200 OK` — Single character feat object.

**Errors:**
- `404` — Feat not found

---

### `DELETE /characters/{id}/feats/{feat_id}`

Remove a feat from a character.

**Path Parameters:**

| Param | Type | Description |
|---|---|---|
| `id` | UUID | Character ID |
| `feat_id` | integer | Feat FK (not the character_feats row ID) |

**Response:** `204 No Content`

---

## Character Spells (Auth Required)

### `GET /characters/{id}/spells`

List all spells for a character.

**Response:** `200 OK`
```json
[
  {
    "character_id": "550e8400-...",
    "spell_id": 12,
    "is_prepared": true
  }
]
```

---

### `POST /characters/{id}/spells`

Add a spell to a character.

**Request Body:**
```json
{
  "spell_id": 12,
  "is_prepared": false
}
```

| Field | Type | Required | Default | Description |
|---|---|---|---|---|
| `spell_id` | integer | Yes | — | FK to spells table |
| `is_prepared` | boolean | No | `false` | Whether spell is prepared |

**Response:** `200 OK` — Single character spell object.

---

### `PUT /characters/{id}/spells/{spell_id}`

Toggle prepared status for a spell.

**Path Parameters:**

| Param | Type | Description |
|---|---|---|
| `id` | UUID | Character ID |
| `spell_id` | integer | Spell FK |

**Request Body:**
```json
{
  "is_prepared": true
}
```

**Response:** `200 OK` — Updated character spell object.

**Errors:**
- `404` — Character spell not found

---

### `DELETE /characters/{id}/spells/{spell_id}`

Remove a spell from a character.

**Response:** `204 No Content`

---

## Character Inventory (Auth Required)

### `GET /characters/{id}/inventory`

List all inventory items for a character.

**Response:** `200 OK`
```json
[
  {
    "id": 1,
    "character_id": "550e8400-...",
    "item_id": 42,
    "quantity": 1,
    "is_equipped": true,
    "is_attuned": false,
    "notes": null
  }
]
```

---

### `POST /characters/{id}/inventory`

Add an item to a character's inventory.

**Request Body:**
```json
{
  "item_id": 42,
  "quantity": 1,
  "is_equipped": true,
  "is_attuned": false,
  "notes": "Starting equipment"
}
```

| Field | Type | Required | Default | Description |
|---|---|---|---|---|
| `item_id` | integer | Yes | — | FK to items table |
| `quantity` | integer | No | `1` | Stack count |
| `is_equipped` | boolean | No | `false` | Whether item is equipped |
| `is_attuned` | boolean | No | `false` | Whether item requires/has attunement active |
| `notes` | string | No | null | Player notes for this item |

**Response:** `200 OK` — Single inventory item object.

---

### `PUT /characters/{id}/inventory/{inventory_id}`

Update an inventory item. **Partial update** — only provided fields are changed, omitted fields keep their current value.

**Path Parameters:**

| Param | Type | Description |
|---|---|---|
| `id` | UUID | Character ID |
| `inventory_id` | integer | Inventory row ID (not item_id) |

**Request Body:**
```json
{
  "quantity": 3,
  "is_equipped": false,
  "is_attuned": null,
  "notes": null
}
```

| Field | Type | Required | Description |
|---|---|---|---|
| `quantity` | integer | No | New quantity (null = keep current) |
| `is_equipped` | boolean | No | New equipped state (null = keep current) |
| `is_attuned` | boolean | No | New attuned state (null = keep current) |
| `notes` | string | No | New notes (null = keep current) |

**Response:** `200 OK` — Updated inventory item object.

**Errors:**
- `404` — Inventory item not found

---

### `DELETE /characters/{id}/inventory/{inventory_id}`

Remove an item from inventory.

**Path Parameters:**

| Param | Type | Description |
|---|---|---|
| `id` | UUID | Character ID |
| `inventory_id` | integer | Inventory row ID |

**Response:** `204 No Content`

---

## Resource Tracking (Auth Required)

### `PATCH /characters/{id}/death-saves`

Update death save successes or failures.

**Path Parameters:**

| Param | Type | Description |
|---|---|---|
| `id` | UUID | Character ID |

**Request Body:**
```json
{
  "successes": 1,
  "failures": 0
}
```

**Response:** `200 OK` — Updated character object.

---

### `PATCH /characters/{id}/spell-slots/{level}`

Update expended spell slots for a specific spell level.

**Path Parameters:**

| Param | Type | Description |
|---|---|---|
| `id` | UUID | Character ID |
| `level` | integer | Spell slot level (1-9) |

**Request Body:**
```json
{
  "expended": 2
}
```

**Response:** `200 OK` — Updated character spell slot object.

---

### `PATCH /characters/{id}/hit-dice/{size}`

Update expended hit dice for a specific die size.

**Path Parameters:**

| Param | Type | Description |
|---|---|---|
| `id` | UUID | Character ID |
| `size` | integer | Die size (6, 8, 10, or 12) |

**Request Body:**
```json
{
  "expended": 1
}
```

**Response:** `200 OK` — Updated character hit dice object.

---

### `PATCH /characters/{id}/features/{feat_id}`

Update the remaining uses for a specific feature.

**Path Parameters:**

| Param | Type | Description |
|---|---|---|
| `id` | UUID | Character ID |
| `feat_id` | integer | Feat FK |

**Request Body:**
```json
{
  "uses_remaining": 0
}
```

**Response:** `200 OK` — Updated character feat object.

---

## Resting (Auth Required)

### `POST /characters/{id}/short-rest`

Perform a short rest, optionally spending hit dice to heal. Resets features that recharge on a short rest.

**Path Parameters:**

| Param | Type | Description |
|---|---|---|
| `id` | UUID | Character ID |

**Request Body:**
```json
{
  "hit_dice_spent": {
    "8": 1,
    "10": 0
  }
}
```

**Response:** `200 OK` — Updated character object.

---

### `POST /characters/{id}/long-rest`

Perform a long rest. Fully heals the character, resets all spell slots, resets features that recharge on a short/long rest, and resets hit dice to 0 expended.

**Path Parameters:**

| Param | Type | Description |
|---|---|---|
| `id` | UUID | Character ID |

**Request Body:** None

**Response:** `200 OK` — Updated character object.

---

## Admin (No Auth)

These endpoints have **no authentication**. Secure them in production.

### `POST /import`

Bulk import D&D data from a 5etools-format JSON file. Supports: classes, class features, subclasses, subclass features, races, subraces, backgrounds, spells, items, monsters, feats, optional features.

**Request Body:** The raw JSON content of a 5etools data file (e.g. `class-paladin.json`, `races.json`, or a homebrew file).

**Request Headers:**
```
Content-Type: application/json
```

**Response:** `200 OK` (empty body on success)

**Notes:**
- All inserts use upsert (`ON CONFLICT ... DO UPDATE`), safe to re-run.
- Body size limit: **5 MB** (configured in main.rs).
- Import order within a single file is handled automatically.
- For multi-file imports, import in this order:
  1. Class files (`class/*.json`)
  2. Core data (`races.json`, `backgrounds.json`, `feats.json`, `optionalfeatures.json`)
  3. Spell files (`spells/*.json`)
  4. Item files (`items.json`, `items-base.json`)
  5. Monster files (`bestiary/*.json`)
  6. Homebrew files

---

### `POST /import/spell-classes`

Import spell-to-class mappings from the `spells/sources.json` file. This populates the `spell_classes` join table so you can query which spells belong to which class.

**Request Body:** The raw JSON content of `spells/sources.json`.

**Response:** `200 OK` (empty body on success)

**Notes:**
- Must be called **after** spells and classes are already imported.
- Processes both `class` and `classVariant` entries.
- Uses `ON CONFLICT DO NOTHING`, safe to re-run.

---

## Route Summary

| Method | Path | Auth | Description |
|---|---|---|---|
| `GET` | `/check_health` | No | Health check |
| `POST` | `/signup` | No | Create account |
| `POST` | `/login` | No | Authenticate |
| `GET` | `/classes` | No | List classes |
| `GET` | `/classes/{name}/{source}` | No | Class detail |
| `GET` | `/spells` | No | Search spells |
| `GET` | `/items` | No | Search items |
| `GET` | `/monsters` | No | Search monsters |
| `GET` | `/races` | No | Search races |
| `GET` | `/backgrounds` | No | Search backgrounds |
| `GET` | `/optional-features` | No | Search optional features |
| `GET` | `/characters` | Yes | List my characters |
| `POST` | `/characters` | Yes | Create character |
| `GET` | `/characters/{id}` | Yes | Get character |
| `PUT` | `/characters/{id}` | Yes | Update character |
| `DELETE` | `/characters/{id}` | Yes | Delete character |
| `GET` | `/characters/{id}/feats` | Yes | List character feats |
| `POST` | `/characters/{id}/feats` | Yes | Add feat |
| `DELETE` | `/characters/{id}/feats/{feat_id}` | Yes | Remove feat |
| `GET` | `/characters/{id}/spells` | Yes | List character spells |
| `POST` | `/characters/{id}/spells` | Yes | Add spell |
| `PUT` | `/characters/{id}/spells/{spell_id}` | Yes | Toggle prepared |
| `DELETE` | `/characters/{id}/spells/{spell_id}` | Yes | Remove spell |
| `GET` | `/characters/{id}/inventory` | Yes | List inventory |
| `POST` | `/characters/{id}/inventory` | Yes | Add item |
| `PUT` | `/characters/{id}/inventory/{inv_id}` | Yes | Update item |
| `DELETE` | `/characters/{id}/inventory/{inv_id}` | Yes | Remove item |
| `PATCH` | `/characters/{id}/death-saves` | Yes | Update death saves |
| `PATCH` | `/characters/{id}/spell-slots/{level}` | Yes | Update expended spell slots |
| `PATCH` | `/characters/{id}/hit-dice/{size}` | Yes | Update expended hit dice |
| `PATCH` | `/characters/{id}/features/{feat_id}` | Yes | Update feature uses |
| `POST` | `/characters/{id}/short-rest` | Yes | Perform short rest |
| `POST` | `/characters/{id}/long-rest` | Yes | Perform long rest |
| `POST` | `/import` | No | Bulk import data |
| `POST` | `/import/spell-classes` | No | Import spell-class mappings |
