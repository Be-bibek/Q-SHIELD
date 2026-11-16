use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use qshield_api::{create_router, state::AppState};
use serde_json::{json, Value};
use tower::ServiceExt;

#[tokio::test]
async fn test_api_endpoints() {
    let state = AppState::new();
    let app = create_router(state);

    // 1. POST /api/v1/images
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/images")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "image_id": "test_img_api",
                "width": 128,
                "height": 128
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
    let val: Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(val["image"]["image_id"], "test_img_api");

    // 2. POST /api/v1/transfers
    let req2 = Request::builder()
        .method("POST")
        .uri("/api/v1/transfers")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "image_id": "test_img_api",
                "sender_id": "SAT_01",
                "receiver_id": "GS_01",
                "tile_size": 64
            })
            .to_string(),
        ))
        .unwrap();

    let response2 = app.clone().oneshot(req2).await.unwrap();
    assert_eq!(response2.status(), StatusCode::OK);
    let body_bytes2 = response2.into_body().collect().await.unwrap().to_bytes();
    let val2: Value = serde_json::from_slice(&body_bytes2).unwrap();
    let transfer_id = val2["transfer"]["transfer_id"].as_str().unwrap();
    assert!(!transfer_id.is_empty());
    assert!(val2["manifest_signature_length"].as_u64().unwrap() > 0);

    // 3. GET /api/v1/transfers/{id}
    let req3 = Request::builder()
        .method("GET")
        .uri(format!("/api/v1/transfers/{}", transfer_id))
        .body(Body::empty())
        .unwrap();

    let response3 = app.clone().oneshot(req3).await.unwrap();
    assert_eq!(response3.status(), StatusCode::OK);

    // 4. GET /api/v1/contacts
    let req4 = Request::builder()
        .method("GET")
        .uri("/api/v1/contacts")
        .body(Body::empty())
        .unwrap();

    let response4 = app.clone().oneshot(req4).await.unwrap();
    assert_eq!(response4.status(), StatusCode::OK);

    // 5. POST /api/v1/simulation/run
    let req5 = Request::builder()
        .method("POST")
        .uri("/api/v1/simulation/run")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "algorithm": "ALL",
                "num_candidates": 8
            })
            .to_string(),
        ))
        .unwrap();

    let response5 = app.clone().oneshot(req5).await.unwrap();
    assert_eq!(response5.status(), StatusCode::OK);
    let body_bytes5 = response5.into_body().collect().await.unwrap().to_bytes();
    let val5: Value = serde_json::from_slice(&body_bytes5).unwrap();
    let results = val5.as_array().unwrap();
    assert_eq!(results.len(), 6); // FIFO, EDF, ImagePriority, ContactAware, SecurityAware, SAViCS
}
