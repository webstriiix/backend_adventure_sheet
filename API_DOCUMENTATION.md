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
    "spell_slots": [[2,0,0,0,0],[2,0,0,0,0],...],
    "additional_spells": [{"prepared":{"2":["divine smite|xphb"],"5":["find steed|xphb"]}}],
    "subclass_title": "Sacred Oath",
    "edition": null
  }
]
```

**Fields:**
- `class_table` — Raw `classTableGroups` from the source JSON (spell slot tables, channel divinity progression, etc.)
- `spell_slots` — Extracted `rowsSpellProgression` array; a 2D array of `[level][spell_level]` where `[character_level-1]` gives max slots per spell level (1–9 for full casters, 1–5 for half casters). `null` for non-spellcasters. Use this instead of hardcoding spell slot numbers.
- `additional_spells` — Always-prepared/known/innate spells granted by class features (e.g., Paladin's Divine Smite at level 2, Find Steed at level 5). These don't count against the spells-prepared limit.

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
    "spell_slots": [[2,0,0,0,0],[2,0,0,0,0],...],
    "additional_spells": null,
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
        "fluff_image_url": null,
        "additional_spells": [{"prepared": {"3":["protection from evil and good","sanctuary"],"5":["lesser restoration","zone of truth"],...}}]
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

**Notes:**
- `class.spell_slots` is a 2D JSON array: `spell_slots[character_level - 1]` gives the max spell slots for that level (e.g., `spell_slots[6]` for level 7).
- `class.additional_spells` lists class-granted spells that are always prepared/known (don't count against limit).
- Each `subclass.additional_spells` lists subclass-granted always-prepared spells (domain/oath/circle spells).

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
    "name": "Sage",
    "source_id": 1,
    "skill_proficiencies": [{"arcana": true, "history": true}],
    "tool_proficiencies": [{"calligrapher's supplies": true}],
    "language_count": 0,
    "starting_equipment": {},
    "ability_bonuses": [
      {"choose": {"weighted": {"from": ["con","int","wis"], "weights": [2, 1]}}},
      {"choose": {"weighted": {"from": ["con","int","wis"], "weights": [1, 1, 1]}}}
    ],
    "grants_bonus_feat": true,
    "granted_feat_id": 42,
    "entries": []
  }
]
```

**Fields:**
- `ability_bonuses` — Weighted choice format. Each element is a `"choose"` block with a `"weighted"` distribution. The player picks between the options (e.g. +2/+1 spread across two abilities, or +1/+1/+1 across all three). Abilities listed in `"from"` are the allowed pool.
- `granted_feat_id` — FK to the `feats` table, populated automatically from `{@feat ...}` references in the background's entries during import. `null` if the background grants no feat.

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
  "max_hp": 12,
  "bonus_feat_id": null,
  "background_feat_id": null
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
| `bonus_feat_id` | integer | No | Feat ID to add if race grants one |
| `background_feat_id` | integer | No | Feat ID to add if background grants one |

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

**Request Body:** Same general shape as create, with additions:

```json
{
  "name": "Guan Yu",
  "class_id": 17,
  "subclass_id": 192,
  "race_id": 1,
  "subrace_id": null,
  "background_id": 1,
  "str": 16,
  "dex": 10,
  "con": 14,
  "int": 8,
  "wis": 12,
  "cha": 15,
  "max_hp": 94,
  "current_hp": 94,
  "temp_hp": 0,
  "experience_pts": 23000,
  "inspiration": false,
  "notes": null,
  "death_saves_successes": null,
  "death_saves_failures": null,
  "cp": null,
  "sp": null,
  "ep": null,
  "gp": null,
  "pp": null
}
```

| Field | Type | Required | Description |
|---|---|---|---|
| `name` | string | Yes | Character name |
| `class_id` | integer | No | Set/change primary class. Inserts or updates the `character_classes` row and marks it as primary. |
| `subclass_id` | integer | No | Set/change subclass on the primary class. Validates the subclass belongs to that class and the level requirement is met. |
| `experience_pts` | integer | Yes | Total XP |
| `race_id` | integer | No | Race FK |
| `subrace_id` | integer | No | Subrace FK |
| `background_id` | integer | No | Background FK |
| `str`-`cha` | integer | Yes | Ability scores |
| `max_hp` | integer | Yes | Max hit points |
| `current_hp` | integer | Yes | Current hit points |
| `temp_hp` | integer | Yes | Temporary hit points (default `0`) |
| `inspiration` | boolean | No | Inspiration flag (null = keep current) |
| `notes` | string | No | Notes (null = keep current) |
| `death_saves_successes` | integer | No | Death save successes (null = keep current) |
| `death_saves_failures` | integer | No | Death save failures (null = keep current) |
| `cp`-`pp` | integer | No | Currency amounts (null = keep current) |

**Response:** `200 OK` — Updated character object.

**Errors:**
- `400` — Subclass not found for this class, or level requirement not met
- `400` — Character has no primary class (when setting subclass)
- `404` — Character not found or access denied

**Notes:**
- `class_id` and `subclass_id` can be used together to change both class and subclass in one call.
- If you only want to change the subclass, send `"subclass_id": <id>` without `class_id`.
- For multiclass level/subclass adjustments, use `PATCH /characters/{id}/classes/{class_id}` instead.

---

### `DELETE /characters/{id}`

Delete a character and all related data (feats, spells, inventory cascade).

**Response:** `204 No Content`

**Errors:**
- `404` — Character not found or access denied

---

### `GET /characters/{id}/actions`

Aggregates all combat actions for a character into D&D Beyond-style buckets (all, attack, action, bonus_action, reaction, other, limited_use).

**Path Parameters:**

| Param | Type | Description |
|---|---|---|
| `id` | UUID | Character ID |

**Response:** `200 OK`
```json
{
  "all": [],
  "attack": [
    {
      "name": "Longsword",
      "source": null,
      "description": "[\"V\"]",
      "range": null,
      "hit_bonus": "+5",
      "damage": "1d8 + 3",
      "max_uses": null,
      "current_uses": null,
      "reset_type": null,
      "time": [{"number": 1, "unit": "action"}]
    }
  ],
  "action": [],
  "bonus_action": [],
  "reaction": [],
  "other": [],
  "limited_use": []
}
```

---

## Character Classes & Leveling (Auth Required)

### `GET /characters/{id}/classes`

List all classes for a character, including subclass info.

**Path Parameters:**

| Param | Type | Description |
|---|---|---|
| `id` | UUID | Character ID |

**Response:** `200 OK`
```json
[
  {
    "class_id": 17,
    "class_name": "Paladin",
    "class_source": "XPHB",
    "level": 3,
    "is_primary": true,
    "subclass_id": 192,
    "subclass_name": "Oath of Glory",
    "subclass_short_name": "Glory",
    "subclass_source": "XPHB"
  }
]
```

**Notes:**
- The primary class is listed first.
- `subclass_*` fields are `null` until a subclass is assigned via `PATCH`.
- Use `class_name` + `class_source` to call `GET /classes/{name}/{source}` and get all class/subclass features.
- Then filter the subclass's `features[]` by `level <= character's level` to get the features available at that level.

**Errors:**
- `404` — Character not found or not owned by user

---

### `POST /characters/{id}/classes`

Add a new class to a character (Multiclassing). Validates that the character meets the ability score prerequisites for the new class.

**Path Parameters:**

| Param | Type | Description |
|---|---|---|
| `id` | UUID | Character ID |

**Request Body:**
```json
{
  "class_id": 2
}
```

**Response:** `200 OK` — Updated character object.

**Errors:**
- `400` — Character does not meet multiclass requirements (e.g., requires 13 CHA).

---

### `PATCH /characters/{id}/classes/{class_id}`

Update a character's level in a specific class and optionally assign a subclass. Validates that the total character level does not exceed 20, and that the subclass unlock level requirement has been met.

**Path Parameters:**

| Param | Type | Description |
|---|---|---|
| `id` | UUID | Character ID |
| `class_id` | integer | Class FK |

**Request Body:**
```json
{
  "level": 3,
  "subclass_id": 5
}
```

**Response:** `200 OK` — Updated character object.

**Errors:**
- `400` — Total character level exceeds 20.
- `400` — Subclass level requirement not met (e.g., subclass unlocks at level 3, but `level` provided is 2).

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

### `GET /characters/{id}/available-feats`

Get a list of feats that are available for the character to choose from. Filters out feats whose prerequisites are not met (e.g., ability score requirements, class level, race, or spellcasting capability).

**Path Parameters:**

| Param | Type | Description |
|---|---|---|
| `id` | UUID | Character ID |

**Response:** `200 OK` — Array of available `Feat` objects.

---

### `POST /characters/{id}/asi-choice`

Apply an Ability Score Improvement (ASI) or choose a Feat for a given character level-up.

**Path Parameters:**

| Param | Type | Description |
|---|---|---|
| `id` | UUID | Character ID |

**Request Body:**
```json
{
  "bump_str": 0,
  "bump_dex": 2,
  "bump_con": 0,
  "bump_int": 0,
  "bump_wis": 1,
  "bump_cha": 0,
  "feat_id": null,
  "source_type": "level"
}
```

*Note: You must either provide a set of `bump_*` properties whose sum total is <= 2, OR provide a `feat_id`. If `feat_id` is provided, the `bump_*` properties are ignored.*

| Field | Type | Required | Default | Description |
|---|---|---|---|---|
| `bump_str` | integer | No | `0` | Bonus to Strength (+1 or +2) |
| `bump_dex` | integer | No | `0` | Bonus to Dexterity (+1 or +2) |
| `bump_con` | integer | No | `0` | Bonus to Constitution (+1 or +2) |
| `bump_int` | integer | No | `0` | Bonus to Intelligence (+1 or +2) |
| `bump_wis` | integer | No | `0` | Bonus to Wisdom (+1 or +2) |
| `bump_cha` | integer | No | `0` | Bonus to Charisma (+1 or +2) |
| `feat_id` | integer | No | `null` | Choose a feat instead of ASI |
| `source_type` | string | No | `"asi"` | Context of the feat/ASI choice |

**Response:** `200 OK` — Updated character object.

**Errors:**
- `400` — Cannot increase ability scores by more than 2
- `404` — Feat or Character not found

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

### `GET /characters/{id}/spell-slots`

Get all spell slots for a character that have been expended.

**Path Parameters:**

| Param | Type | Description |
|---|---|---|
| `id` | UUID | Character ID |

**Response:** `200 OK`
```json
[
  {
    "character_id": "550e8400-...",
    "slot_level": 1,
    "expended": 2
  }
]
```

---

### `GET /characters/{id}/spell-slots/{level}`

Get the expended spell slot status for a specific spell level.

**Path Parameters:**

| Param | Type | Description |
|---|---|---|
| `id` | UUID | Character ID |
| `level` | integer | Spell slot level (1-9) |

**Response:** `200 OK`
```json
{
  "character_id": "550e8400-...",
  "slot_level": 1,
  "expended": 2
}
```

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

### `PATCH /characters/{id}/resources/{resource_name}`

Update the remaining uses for a generic dynamic resource pool (e.g., class features or custom pools).

**Path Parameters:**

| Param | Type | Description |
|---|---|---|
| `id` | UUID | Character ID |
| `resource_name` | string | Name of the resource pool |

**Request Body:**
```json
{
  "uses_remaining": 2
}
```

**Response:** `200 OK` — Updated resource pool object.

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
| `GET` | `/characters/{id}/actions` | Yes | Get character actions |
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
| `GET` | `/characters/{id}/classes` | Yes | List character classes with subclass info |
| `POST` | `/characters/{id}/classes` | Yes | Add class (multiclass) |
| `PATCH` | `/characters/{id}/classes/{class_id}` | Yes | Update class level/subclass |
| `PATCH` | `/characters/{id}/death-saves` | Yes | Update death saves |
| `GET` | `/characters/{id}/spell-slots` | Yes | List expended spell slots |
| `GET` | `/characters/{id}/spell-slots/{level}` | Yes | Get expended spell slot by level |
| `PATCH` | `/characters/{id}/spell-slots/{level}` | Yes | Update expended spell slots |
| `PATCH` | `/characters/{id}/hit-dice/{size}` | Yes | Update expended hit dice |
| `PATCH` | `/characters/{id}/features/{feat_id}` | Yes | Update feature uses |
| `PATCH` | `/characters/{id}/resources/{resource_name}` | Yes | Update generic resource pool uses |
| `POST` | `/characters/{id}/short-rest` | Yes | Perform short rest |
| `POST` | `/characters/{id}/long-rest` | Yes | Perform long rest |
| `GET` | `/characters/{id}/available-feats` | Yes | Get feats available to character |
| `POST` | `/characters/{id}/asi-choice` | Yes | Increase ability scores or pick feat |
| `GET` | `/characters/{id}/proficiencies` | Yes | List character proficiencies |
| `POST` | `/characters/{id}/proficiencies` | Yes | Add/update proficiency or expertise |
| `PATCH` | `/characters/{id}/proficiencies/{prof_id}` | Yes | Change proficiency type |
| `DELETE` | `/characters/{id}/proficiencies/{prof_id}` | Yes | Remove proficiency |
| `POST` | `/import` | No | Bulk import data |
| `POST` | `/import/spell-classes` | No | Import spell-class mappings |

---

## Race Options (Frontend contract)

These endpoints expose race/subrace selectable options (cantrips, bonus feats, variable traits) and allow saving a character's selections.

### `GET /races/{name}/{source}/options`

Response 200 (array of race options):
```json
[
  {
    "id": 42,
    "race_id": 3,
    "subrace_id": null,
    "source_id": 1,
    "option_type": "cantrip",
    "choices": [{"name":"shillelagh"},{"name":"minor illusion"}],
    "min_choose": 1,
    "max_choose": 1,
    "note": "Pick one cantrip"
  }
]
```

Notes:
- `choices` === `null` means free-form choice (frontend must still enforce min/max).
- Frontend should display choices and enforce `min_choose`/`max_choose`.

### `POST /characters/{character_id}/race-options`

Save a player's selection for a race option.

Request body (single choice):
```json
{ "race_option_id": 42, "selection": { "name": "shillelagh" } }
```
Multi-choice example:
```json
{ "race_option_id": 99, "selection": [{"name":"mending"},{"name":"guidance"}] }
```

Response 200 (persisted selection):
```json
{
  "id": 7,
  "character_id": "550e8400-e29b-41d4-a716-446655440000",
  "race_option_id": 42,
  "selection": { "name": "shillelagh" }
}
```

Errors:
- `400` — invalid selection, violates min/max or not in choices
- `401`/`403` — auth/ownership failures
- `404` — race option not found

---

## Character Proficiencies (Auth Required)

### `GET /characters/{id}/proficiencies`

List all manually assigned proficiencies and expertise for a character.

**Path Parameters:**

| Param | Type | Description |
|---|---|---|
| `id` | UUID | Character ID |

**Response:** `200 OK`
```json
[
  {
    "id": 1,
    "character_id": "550e8400-e29b-41d4-a716-446655440000",
    "category": "skill",
    "name": "athletics",
    "proficiency_type": "proficiency"
  },
  {
    "id": 2,
    "character_id": "550e8400-...",
    "category": "saving_throw",
    "name": "wisdom",
    "proficiency_type": "expertise"
  }
]
```

---

### `POST /characters/{id}/proficiencies`

Add or update a proficiency/expertise entry. Uses upsert — if the same `(category, name)` already exists for this character, the `proficiency_type` is updated.

**Path Parameters:**

| Param | Type | Description |
|---|---|---|
| `id` | UUID | Character ID |

**Request Body:**
```json
{
  "category": "skill",
  "name": "athletics",
  "proficiency_type": "expertise"
}
```

| Field | Type | Required | Default | Description |
|---|---|---|---|---|
| `category` | string | Yes | — | `"saving_throw"` or `"skill"` |
| `name` | string | Yes | — | Lowercase name (e.g. `"athletics"`, `"wisdom"`) |
| `proficiency_type` | string | No | `"proficiency"` | `"proficiency"` or `"expertise"` |

**Response:** `200 OK` — Single proficiency object.

**Errors:**
- `400` — Invalid category or proficiency_type value

---

### `PATCH /characters/{id}/proficiencies/{prof_id}`

Change the proficiency_type of an existing entry.

**Path Parameters:**

| Param | Type | Description |
|---|---|---|
| `id` | UUID | Character ID |
| `prof_id` | integer | Proficiency row ID |

**Request Body:**
```json
{
  "proficiency_type": "expertise"
}
```

**Response:** `200 OK` — Updated proficiency object.

**Errors:**
- `404` — Character proficiency not found

---

### `DELETE /characters/{id}/proficiencies/{prof_id}`

Remove a proficiency entry.

**Path Parameters:**

| Param | Type | Description |
|---|---|---|
| `id` | UUID | Character ID |
| `prof_id` | integer | Proficiency row ID |

**Response:** `204 No Content`

**Errors:**
- `404` — Character proficiency not found

---

## Class Resources (Frontend contract)

Endpoint returns computed resources like Channel Divinity uses and Lay on Hands pool for a given level.

### `GET /classes/{name}/{source}/resources/{level}`

Example: `GET /classes/Paladin/PHB/resources/11`

Response 200:
```json
{
  "class_name":"Paladin",
  "source":"PHB",
  "level": 11,
  "lay_on_hands_pool": 55,
  "channel_divinity_uses": 2,
  "subclass_options": [
    {
      "gate_feature_id": 210,
      "gate_feature_name": "Sacred Oath",
      "choices": [
        {"subclass_feature_id": 501, "name": "Oath of Devotion"},
        {"subclass_feature_id": 502, "name": "Oath of the Ancients"}
      ]
    }
  ]
}
```

Notes:
- `lay_on_hands_pool` is computed as `level * 5` for paladins.
- `channel_divinity_uses` is read from the class `class_table` progression.
- **Spell slots** per level are available from `GET /classes/{name}/{source}` → `class.spell_slots[level-1]`. No need to hardcode them.
- **Free spells** that are always prepared and don't count against the limit are in `class.additional_spells` (class-level) and each subclass's `subclass.additional_spells`. The `prepared` key indicates spells always prepared; `known` for always known; `innate` for innate casting.

---
