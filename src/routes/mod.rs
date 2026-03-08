use crate::{
    db::AppState,
    handlers::{admin, auth, characters, classes, compendium},
};
use axum::{
    Router,
    routing::{delete, get, patch, post, put},
};

async fn check_health() -> &'static str {
    "Server Good"
}

pub fn all_routes() -> Router<AppState> {
    let auth_routes = Router::new()
        .route("/signup", post(auth::signup))
        .route("/login", post(auth::login));

    let class_routes = Router::new()
        .route("/classes", get(classes::list_classes))
        .route("/classes/{name}/{source}", get(classes::get_class_detail));

    let compendium_routes = Router::new()
        .route("/spells", get(compendium::list_spells))
        .route("/items", get(compendium::list_items))
        .route("/feats", get(compendium::list_feats))
        .route("/monsters", get(compendium::list_monsters))
        .route("/races", get(compendium::list_races))
        .route("/backgrounds", get(compendium::list_backgrounds))
        .route(
            "/optional-features",
            get(compendium::list_optional_features),
        );

    let char_routes = Router::new()
        .route(
            "/characters",
            get(characters::list_characters).post(characters::create_character),
        )
        .route(
            "/characters/{id}",
            get(characters::get_character)
                .put(characters::update_character)
                .delete(characters::delete_character),
        )
        // Character classes and leveling
        .route(
            "/characters/{id}/classes",
            post(characters::add_character_class),
        )
        .route(
            "/characters/{id}/classes/{class_id}",
            patch(characters::update_character_class),
        )
        // Character feats
        .route(
            "/characters/{id}/feats",
            get(characters::list_character_feats).post(characters::add_character_feat),
        )
        .route(
            "/characters/{id}/feats/{feat_id}",
            delete(characters::remove_character_feat),
        )
        // Character spells
        .route(
            "/characters/{id}/spells",
            get(characters::list_character_spells).post(characters::add_character_spell),
        )
        .route(
            "/characters/{id}/spells/{spell_id}",
            put(characters::update_character_spell).delete(characters::remove_character_spell),
        )
        // Character inventory
        .route(
            "/characters/{id}/inventory",
            get(characters::list_character_inventory).post(characters::add_inventory_item),
        )
        .route(
            "/characters/{id}/inventory/{inventory_id}",
            put(characters::update_inventory_item).delete(characters::remove_inventory_item),
        )
        // Resource tracking
        .route(
            "/characters/{id}/death-saves",
            patch(characters::update_death_saves),
        )
        .route(
            "/characters/{id}/spell-slots",
            get(characters::get_spell_slots),
        )
        .route(
            "/characters/{id}/spell-slots/{level}",
            get(characters::get_spell_slot).patch(characters::update_spell_slots),
        )
        .route(
            "/characters/{id}/hit-dice/{size}",
            patch(characters::update_hit_dice),
        )
        .route(
            "/characters/{id}/features/{feat_id}",
            patch(characters::update_feature_uses),
        )
        // Generic dynamically-named resources
        .route(
            "/characters/{id}/resources/{resource_name}",
            patch(characters::update_resource_uses),
        )
        // Rests
        .route("/characters/{id}/short-rest", post(characters::short_rest))
        .route("/characters/{id}/long-rest", post(characters::long_rest))
        // ASI & Feats
        .route(
            "/characters/{id}/available-feats",
            get(characters::list_available_feats),
        )
        .route(
            "/characters/{id}/asi-choice",
            post(characters::choose_asi_or_feat),
        )
        // Actions
        .route(
            "/characters/{id}/actions",
            get(characters::get_character_actions),
        );

    let admin_routes = Router::new()
        .route("/import", post(admin::trigger_import))
        .route(
            "/import/spell-classes",
            post(admin::trigger_import_spell_classes),
        );

    Router::new()
        .route("/check_health", get(check_health))
        .merge(auth_routes)
        .merge(class_routes)
        .merge(compendium_routes)
        .merge(char_routes)
        .merge(admin_routes)
}
