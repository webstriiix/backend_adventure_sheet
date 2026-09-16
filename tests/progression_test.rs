mod common;

use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use common::{TEST_JWT_SECRET, TestApp, bearer, create_test_character, create_test_user};
use serde_json::{Value, json};
use sqlx::PgPool;
use tower::ServiceExt;
use uuid::Uuid;

async fn setup_test_wizard_class(pool: &PgPool) -> (i32, i32) {
    let source_id = sqlx::query_scalar!(
        r#"
        INSERT INTO sources (slug, full_name, is_homebrew)
        VALUES ('TEST_WIZ_SRC', 'Test Wizard Source', false)
        ON CONFLICT (slug) DO UPDATE SET full_name = EXCLUDED.full_name
        RETURNING id
        "#
    )
    .fetch_one(pool)
    .await
    .expect("Failed to insert source");

    let class_id = sqlx::query_scalar!(
        r#"
        INSERT INTO classes (name, source_id, hit_die, asi_levels)
        VALUES ('TestWizard', $1, 6, ARRAY[4, 8, 12, 16, 19]::integer[])
        ON CONFLICT (name, source_id) DO UPDATE SET asi_levels = EXCLUDED.asi_levels
        RETURNING id
        "#,
        source_id
    )
    .fetch_one(pool)
    .await
    .expect("Failed to insert class");

    // Insert subclass feature (subclass gate at level 3)
    let _ = sqlx::query!(
        r#"
        INSERT INTO class_features (name, source_id, class_id, level, entries, is_subclass_gate)
        VALUES ('Subclass Feature Gate', $1, $2, 3, '[]'::jsonb, true)
        ON CONFLICT DO NOTHING
        "#,
        source_id,
        class_id
    )
    .execute(pool)
    .await;

    // Insert a subclass
    let subclass_id = sqlx::query_scalar!(
        r#"
        INSERT INTO subclasses (name, short_name, source_id, class_id, unlock_level)
        VALUES ('School of Evocation', 'Evocation', $1, $2, 3)
        ON CONFLICT DO NOTHING
        RETURNING id
        "#,
        source_id,
        class_id
    )
    .fetch_optional(pool)
    .await
    .expect("Failed to query subclass")
    .unwrap_or(1);

    (class_id, subclass_id)
}

async fn assign_class_level(
    pool: &PgPool,
    char_id: Uuid,
    class_id: i32,
    level: i32,
    subclass_id: Option<i32>,
) {
    sqlx::query!(
        r#"
        INSERT INTO character_classes (character_id, class_id, level, is_primary, subclass_id)
        VALUES ($1, $2, $3, true, $4)
        ON CONFLICT (character_id, class_id) DO UPDATE SET
            level = EXCLUDED.level,
            is_primary = true,
            subclass_id = EXCLUDED.subclass_id
        "#,
        char_id,
        class_id,
        level,
        subclass_id
    )
    .execute(pool)
    .await
    .expect("Failed to assign class level");
}

#[tokio::test]
async fn test_progression_level_1_has_full_level_20_roadmap_with_locked_slots() {
    let app = TestApp::new().await;
    let (user_id, token) = create_test_user(&app.pool, "prog_u1", TEST_JWT_SECRET).await;
    let char_id = create_test_character(&app.pool, user_id, "WizLevel1").await;
    let (class_id, _) = setup_test_wizard_class(&app.pool).await;
    assign_class_level(&app.pool, char_id, class_id, 1, None).await;

    let response = app
        .router
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/characters/{}/progression", char_id))
                .header("authorization", bearer(&token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
    let manifest: Value = serde_json::from_slice(&bytes).unwrap();

    assert_eq!(manifest["total_level"], 1);
    assert_eq!(manifest["class_name"], "TestWizard");

    let decision_points = manifest["decision_points"]
        .as_array()
        .expect("decision_points array");

    // Check Subclass at Level 3 -> must be locked
    let sc_dp = decision_points
        .iter()
        .find(|dp| dp["level"] == 3 && dp["choice_type"] == "subclass")
        .expect("Subclass decision point at level 3 should exist in roadmap");
    assert_eq!(sc_dp["status"], "locked");

    // Check ASI slots at Level 4, 8, 12, 16, 19 -> all must be locked
    for asi_lvl in [4, 8, 12, 16, 19] {
        let asi_dp = decision_points
            .iter()
            .find(|dp| dp["level"] == asi_lvl && dp["choice_type"] == "asi")
            .unwrap_or_else(|| panic!("ASI decision point at level {} should exist", asi_lvl));
        assert_eq!(
            asi_dp["status"], "locked",
            "ASI at level {} should be locked for level 1 character",
            asi_lvl
        );
    }
}

#[tokio::test]
async fn test_progression_level_4_unlocks_level_3_and_4_slots() {
    let app = TestApp::new().await;
    let (user_id, token) = create_test_user(&app.pool, "prog_u2", TEST_JWT_SECRET).await;
    let char_id = create_test_character(&app.pool, user_id, "WizLevel4").await;
    let (class_id, _subclass_id) = setup_test_wizard_class(&app.pool).await;

    // Character is Level 4 with no subclass or ASI choices chosen yet
    assign_class_level(&app.pool, char_id, class_id, 4, None).await;

    let response = app
        .router
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/characters/{}/progression", char_id))
                .header("authorization", bearer(&token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
    let manifest: Value = serde_json::from_slice(&bytes).unwrap();

    let decision_points = manifest["decision_points"].as_array().unwrap();

    // Subclass at Level 3 should be "pending" (unlocked because total_level=4, but none chosen)
    let sc_dp = decision_points
        .iter()
        .find(|dp| dp["level"] == 3 && dp["choice_type"] == "subclass")
        .expect("Subclass DP should exist");
    assert_eq!(sc_dp["status"], "pending");

    // ASI at Level 4 should be "pending"
    let asi_4 = decision_points
        .iter()
        .find(|dp| dp["level"] == 4 && dp["choice_type"] == "asi")
        .expect("ASI 4 DP should exist");
    assert_eq!(asi_4["status"], "pending");

    // ASI at Level 8, 12, 16, 19 should still be "locked"
    for asi_lvl in [8, 12, 16, 19] {
        let asi_dp = decision_points
            .iter()
            .find(|dp| dp["level"] == asi_lvl && dp["choice_type"] == "asi")
            .unwrap();
        assert_eq!(asi_dp["status"], "locked");
    }

    // Now fill ASI at level 4 and Subclass
    let _ = app
        .router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/characters/{}/asi-choice", char_id))
                .header("content-type", "application/json")
                .header("authorization", bearer(&token))
                .body(Body::from(
                    json!({
                        "bump_int": 2,
                        "gained_at_level": 4
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    // Set subclass
    assign_class_level(&app.pool, char_id, class_id, 4, Some(_subclass_id)).await;

    // Re-fetch progression
    let res2 = app
        .router
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/characters/{}/progression", char_id))
                .header("authorization", bearer(&token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let bytes2 = to_bytes(res2.into_body(), 1024 * 1024).await.unwrap();
    let manifest2: Value = serde_json::from_slice(&bytes2).unwrap();
    let dps2 = manifest2["decision_points"].as_array().unwrap();

    let sc_dp2 = dps2
        .iter()
        .find(|dp| dp["level"] == 3 && dp["choice_type"] == "subclass")
        .unwrap();
    assert_eq!(sc_dp2["status"], "complete");

    let asi_4_2 = dps2
        .iter()
        .find(|dp| dp["level"] == 4 && dp["choice_type"] == "asi")
        .unwrap();
    assert_eq!(asi_4_2["status"], "complete");

    // ASI 8 remains locked
    let asi_8_2 = dps2
        .iter()
        .find(|dp| dp["level"] == 8 && dp["choice_type"] == "asi")
        .unwrap();
    assert_eq!(asi_8_2["status"], "locked");
}
