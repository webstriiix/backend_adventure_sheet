mod common;

use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use common::TestApp;
use serde_json::Value;
use tower::ServiceExt;

#[tokio::test]
async fn test_list_feats_includes_source_slug() {
    let app = TestApp::new().await;

    // Seed test sources & feats (e.g. Alert for PHB and XPHB)
    let src_phb_id = sqlx::query_scalar!(
        r#"
        INSERT INTO sources (slug, full_name, is_homebrew)
        VALUES ('PHB', 'Player Handbook 2014', false)
        ON CONFLICT (slug) DO UPDATE SET full_name = EXCLUDED.full_name
        RETURNING id
        "#
    )
    .fetch_one(&app.pool)
    .await
    .expect("insert PHB source");

    let src_xphb_id = sqlx::query_scalar!(
        r#"
        INSERT INTO sources (slug, full_name, is_homebrew)
        VALUES ('XPHB', 'Player Handbook 2024', false)
        ON CONFLICT (slug) DO UPDATE SET full_name = EXCLUDED.full_name
        RETURNING id
        "#
    )
    .fetch_one(&app.pool)
    .await
    .expect("insert XPHB source");

    sqlx::query!(
        r#"
        INSERT INTO feats (name, source_id, entries)
        VALUES ('Alert', $1, '["PHB 2014 Alert"]'::jsonb)
        ON CONFLICT (name, source_id) DO UPDATE SET entries = EXCLUDED.entries
        "#,
        src_phb_id
    )
    .execute(&app.pool)
    .await
    .expect("insert Alert PHB");

    sqlx::query!(
        r#"
        INSERT INTO feats (name, source_id, entries)
        VALUES ('Alert', $1, '["XPHB 2024 Alert"]'::jsonb)
        ON CONFLICT (name, source_id) DO UPDATE SET entries = EXCLUDED.entries
        "#,
        src_xphb_id
    )
    .execute(&app.pool)
    .await
    .expect("insert Alert XPHB");

    // GET /feats?name=Alert
    let response = app
        .router
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/feats?name=Alert")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
    let feats: Vec<Value> = serde_json::from_slice(&bytes).unwrap();

    assert!(feats.len() >= 2);
    let phb_feat = feats
        .iter()
        .find(|f| f["source_slug"] == "PHB")
        .expect("PHB Alert feat");
    assert_eq!(phb_feat["name"], "Alert");
    assert_eq!(phb_feat["source_slug"], "PHB");

    let xphb_feat = feats
        .iter()
        .find(|f| f["source_slug"] == "XPHB")
        .expect("XPHB Alert feat");
    assert_eq!(xphb_feat["name"], "Alert");
    assert_eq!(xphb_feat["source_slug"], "XPHB");
}
