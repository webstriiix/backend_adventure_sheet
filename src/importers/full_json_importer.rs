pub use super::import_spells::import_spell_classes;
use super::{
    import_backgrounds::import_backgrounds, import_classes::import_classes,
    import_feats::import_feats, import_items::import_items, import_monsters::import_monsters,
    import_optional_features::import_optional_features, import_races::import_races,
    import_spells::import_spells,
};
use serde_json::Value;
use sqlx::PgPool;

pub async fn import_everything(pool: &PgPool, raw: &str) -> anyhow::Result<()> {
    let data: Value = serde_json::from_str(raw)?;

    // 1. Feats (before classes so feat references resolve)
    import_feats(pool, &data).await?;

    // 2. Classes, class features, subclasses, subclass features
    import_classes(pool, &data).await?;

    // 3. Races and subraces
    import_races(pool, &data).await?;

    // 4. Backgrounds
    import_backgrounds(pool, &data).await?;

    // 5. Spells
    import_spells(pool, &data).await?;

    // 6. Items and base items
    import_items(pool, &data).await?;

    // 7. Monsters
    import_monsters(pool, &data).await?;

    // 8. Optional features
    import_optional_features(pool, &data).await?;

    Ok(())
}
