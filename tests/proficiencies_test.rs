mod common;

use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use common::{TEST_JWT_SECRET, TestApp, bearer, create_test_character, create_test_user};
use serde_json::{Value, json};
use tower::ServiceExt;

#[tokio::test]
async fn test_add_skill_ok() {
    let app = TestApp::new().await;
    let (user_id, token) = create_test_user(&app.pool, "prof_u1", TEST_JWT_SECRET).await;
    let char_id = create_test_character(&app.pool, user_id, "ProfChar1").await;

    let response = app
        .router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/characters/{}/proficiencies", char_id))
                .header("content-type", "application/json")
                .header("authorization", bearer(&token))
                .body(Body::from(
                    json!({
                        "category": "skill",
                        "name": "Stealth",
                        "proficiency_type": "proficiency"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
    let body: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(body["name"], "stealth");
    assert_eq!(body["category"], "skill");
    assert_eq!(body["proficiency_type"], "proficiency");
}

#[tokio::test]
async fn test_duplicate_skill_returns_409() {
    let app = TestApp::new().await;
    let (user_id, token) = create_test_user(&app.pool, "prof_u2", TEST_JWT_SECRET).await;
    let char_id = create_test_character(&app.pool, user_id, "ProfChar2").await;

    // First POST: Should succeed (200)
    let res1 = app
        .router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/characters/{}/proficiencies", char_id))
                .header("content-type", "application/json")
                .header("authorization", bearer(&token))
                .body(Body::from(
                    json!({
                        "category": "skill",
                        "name": "Perception",
                        "proficiency_type": "proficiency"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res1.status(), StatusCode::OK);

    // Second POST: Duplicate skill should return 409 Conflict
    let res2 = app
        .router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/characters/{}/proficiencies", char_id))
                .header("content-type", "application/json")
                .header("authorization", bearer(&token))
                .body(Body::from(
                    json!({
                        "category": "skill",
                        "name": "Perception",
                        "proficiency_type": "proficiency"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res2.status(), StatusCode::CONFLICT);
    let bytes = to_bytes(res2.into_body(), 1024 * 1024).await.unwrap();
    let body: Value = serde_json::from_slice(&bytes).unwrap();
    let error_msg = body["error"].as_str().unwrap();
    assert!(
        error_msg.contains("DUPLICATE_SKILL_REPLACEMENT_REQUIRED"),
        "Error should contain DUPLICATE_SKILL_REPLACEMENT_REQUIRED but got: {}",
        error_msg
    );
}

#[tokio::test]
async fn test_batch_add_skills() {
    let app = TestApp::new().await;
    let (user_id, token) = create_test_user(&app.pool, "prof_u3", TEST_JWT_SECRET).await;
    let char_id = create_test_character(&app.pool, user_id, "ProfChar3").await;

    let response = app
        .router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/characters/{}/proficiencies/batch", char_id))
                .header("content-type", "application/json")
                .header("authorization", bearer(&token))
                .body(Body::from(
                    json!([
                        {"category": "skill", "name": "Acrobatics", "proficiency_type": "proficiency"},
                        {"category": "skill", "name": "Athletics", "proficiency_type": "proficiency"},
                        {"category": "saving_throw", "name": "Strength", "proficiency_type": "proficiency"}
                    ])
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
    let body: Value = serde_json::from_slice(&bytes).unwrap();
    let arr = body.as_array().unwrap();
    assert_eq!(arr.len(), 3);
}

#[tokio::test]
async fn test_list_skills_only() {
    let app = TestApp::new().await;
    let (user_id, token) = create_test_user(&app.pool, "prof_u4", TEST_JWT_SECRET).await;
    let char_id = create_test_character(&app.pool, user_id, "ProfChar4").await;

    // Add 2 skills and 1 saving throw
    let _ = app
        .router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/characters/{}/proficiencies/batch", char_id))
                .header("content-type", "application/json")
                .header("authorization", bearer(&token))
                .body(Body::from(
                    json!([
                        {"category": "skill", "name": "Insight", "proficiency_type": "proficiency"},
                        {"category": "skill", "name": "History", "proficiency_type": "proficiency"},
                        {"category": "saving_throw", "name": "Wisdom", "proficiency_type": "proficiency"}
                    ])
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    // GET /characters/{id}/proficiencies/skills
    let response = app
        .router
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/characters/{}/proficiencies/skills", char_id))
                .header("authorization", bearer(&token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
    let skills: Vec<String> = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(skills.len(), 2);
    assert!(skills.contains(&"insight".to_string()));
    assert!(skills.contains(&"history".to_string()));
    assert!(!skills.contains(&"wisdom".to_string()));
}

#[tokio::test]
async fn test_invalid_category_400() {
    let app = TestApp::new().await;
    let (user_id, token) = create_test_user(&app.pool, "prof_u5", TEST_JWT_SECRET).await;
    let char_id = create_test_character(&app.pool, user_id, "ProfChar5").await;

    let response = app
        .router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/characters/{}/proficiencies", char_id))
                .header("content-type", "application/json")
                .header("authorization", bearer(&token))
                .body(Body::from(
                    json!({
                        "category": "invalid_category",
                        "name": "Stealth",
                        "proficiency_type": "proficiency"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_unauthorized_no_token_401() {
    let app = TestApp::new().await;
    let (user_id, _) = create_test_user(&app.pool, "prof_u6", TEST_JWT_SECRET).await;
    let char_id = create_test_character(&app.pool, user_id, "ProfChar6").await;

    let response = app
        .router
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/characters/{}/proficiencies", char_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_ownership_denied_404() {
    let app = TestApp::new().await;
    let (user_a, _token_a) = create_test_user(&app.pool, "prof_user_a", TEST_JWT_SECRET).await;
    let (_user_b, token_b) = create_test_user(&app.pool, "prof_user_b", TEST_JWT_SECRET).await;

    // Character belongs to User A
    let char_id_a = create_test_character(&app.pool, user_a, "CharUserA").await;

    // User B tries to access User A's character -> 404
    let response = app
        .router
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/characters/{}/proficiencies", char_id_a))
                .header("authorization", bearer(&token_b))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
