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

async fn setup_test_class_and_feat(pool: &PgPool) -> (i32, i32) {
    let source_id = sqlx::query_scalar!(
        r#"
        INSERT INTO sources (slug, full_name, is_homebrew)
        VALUES ('TEST_SRC', 'Test Source', false)
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
        VALUES ('TestFighter', $1, 10, ARRAY[4, 6, 8, 12, 14, 16, 19]::integer[])
        ON CONFLICT (name, source_id) DO UPDATE SET asi_levels = EXCLUDED.asi_levels
        RETURNING id
        "#,
        source_id
    )
    .fetch_one(pool)
    .await
    .expect("Failed to insert class");

    let feat_id = sqlx::query_scalar!(
        r#"
        INSERT INTO feats (name, source_id, has_uses, recharge_on, entries)
        VALUES ('TestActorFeat', $1, false, NULL, '[]'::jsonb)
        ON CONFLICT (name, source_id) DO UPDATE SET has_uses = EXCLUDED.has_uses
        RETURNING id
        "#,
        source_id
    )
    .fetch_one(pool)
    .await
    .expect("Failed to insert feat");

    (class_id, feat_id)
}

async fn assign_class_to_character(pool: &PgPool, char_id: Uuid, class_id: i32, level: i32) {
    sqlx::query!(
        r#"
        INSERT INTO character_classes (character_id, class_id, level, is_primary)
        VALUES ($1, $2, $3, true)
        ON CONFLICT (character_id, class_id) DO UPDATE SET level = EXCLUDED.level, is_primary = true
        "#,
        char_id,
        class_id,
        level
    )
    .execute(pool)
    .await
    .expect("Failed to assign class to character");
}

#[tokio::test]
async fn test_asi_history_empty_initial() {
    let app = TestApp::new().await;
    let (user_id, token) = create_test_user(&app.pool, "asi_u1", TEST_JWT_SECRET).await;
    let char_id = create_test_character(&app.pool, user_id, "AsiChar1").await;

    let response = app
        .router
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/characters/{}/asi-history", char_id))
                .header("authorization", bearer(&token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
    let history: Vec<Value> = serde_json::from_slice(&bytes).unwrap();
    assert!(history.is_empty());
}

#[tokio::test]
async fn test_choose_asi_stat_bump() {
    let app = TestApp::new().await;
    let (user_id, token) = create_test_user(&app.pool, "asi_u2", TEST_JWT_SECRET).await;
    let char_id = create_test_character(&app.pool, user_id, "AsiChar2").await;
    let (class_id, _) = setup_test_class_and_feat(&app.pool).await;
    assign_class_to_character(&app.pool, char_id, class_id, 4).await;

    // POST /characters/{id}/asi-choice with bump_str=1, bump_con=1 at level 4
    let response = app
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
                        "bump_str": 1,
                        "bump_con": 1,
                        "gained_at_level": 4
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
    let updated_char: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(updated_char["str"], 11);
    assert_eq!(updated_char["con"], 11);

    // GET /characters/{id}/asi-history should have 1 record with bump_str=1, bump_con=1
    let history_res = app
        .router
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/characters/{}/asi-history", char_id))
                .header("authorization", bearer(&token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(history_res.status(), StatusCode::OK);
    let hist_bytes = to_bytes(history_res.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let history: Vec<Value> = serde_json::from_slice(&hist_bytes).unwrap();
    assert_eq!(history.len(), 1);
    assert_eq!(history[0]["level"], 4);
    assert_eq!(history[0]["bump_str"], 1);
    assert_eq!(history[0]["bump_con"], 1);
    assert_eq!(history[0]["bump_dex"], 0);
}

#[tokio::test]
async fn test_choose_feat_at_asi() {
    let app = TestApp::new().await;
    let (user_id, token) = create_test_user(&app.pool, "asi_u3", TEST_JWT_SECRET).await;
    let char_id = create_test_character(&app.pool, user_id, "AsiChar3").await;
    let (class_id, feat_id) = setup_test_class_and_feat(&app.pool).await;
    assign_class_to_character(&app.pool, char_id, class_id, 4).await;

    // POST /characters/{id}/asi-choice with feat_id at level 4
    let response = app
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
                        "feat_id": feat_id,
                        "gained_at_level": 4
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // GET /characters/{id}/asi-history should have record with feat_id
    let history_res = app
        .router
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/characters/{}/asi-history", char_id))
                .header("authorization", bearer(&token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(history_res.status(), StatusCode::OK);
    let hist_bytes = to_bytes(history_res.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let history: Vec<Value> = serde_json::from_slice(&hist_bytes).unwrap();
    assert_eq!(history.len(), 1);
    assert_eq!(history[0]["level"], 4);
    assert_eq!(history[0]["feat_id"], feat_id);
    assert_eq!(history[0]["bump_str"], 0);
}
