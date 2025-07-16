use std::sync::Arc;

use sentinel_guard::{
    models::{
        access_token::{
            AccessTokenCreatePayloadWithAccessToken, AccessTokenFilter, AccessTokenUpdatePayload,
        },
        pagination::Pagination,
    },
    repositories::{access_token_repository::AccessTokenRepository, base::Repository},
};
use sqlx::PgPool;
use uuid::Uuid;

#[sqlx::test(fixtures("../fixtures/access_tokens.sql"))]
async fn test_access_token_repository_create_with_valid_data_succeeds(pool: PgPool) {
    let repository = AccessTokenRepository::new(Arc::new(pool));
    let payload = AccessTokenCreatePayloadWithAccessToken {
        project_access_id: "00000000-0000-0000-0000-000000000101".to_string(),
        algorithm: "HS512".to_string(),
        expires_at: "2031-01-01T00:00:00Z".to_string(),
        access_token: "test-token".to_string(),
    };
    let access_token = repository.create(payload.clone()).await.unwrap();
    assert_eq!(
        access_token.project_access_id,
        Uuid::parse_str("00000000-0000-0000-0000-000000000101").unwrap()
    );
    assert_eq!(access_token.algorithm, "HS512");
    assert!(access_token.active);
}

#[sqlx::test]
async fn test_access_token_repository_create_with_missing_project_access_id_fails(pool: PgPool) {
    let repository = AccessTokenRepository::new(Arc::new(pool));
    let payload = AccessTokenCreatePayloadWithAccessToken {
        project_access_id: Uuid::new_v4().to_string(),
        algorithm: "HS256".to_string(),
        expires_at: "2031-01-01T00:00:00Z".to_string(),
        access_token: "test-token".to_string(),
    };
    let result = repository.create(payload).await;
    assert!(result.is_err());
}

#[sqlx::test(fixtures("../fixtures/access_tokens.sql"))]
async fn test_access_token_repository_read_existing_succeeds(pool: PgPool) {
    let repository = AccessTokenRepository::new(Arc::new(pool));
    let id = Uuid::parse_str("11111111-1111-1111-1111-111111111111").unwrap();
    let access_token = repository.read(id).await.unwrap();
    assert!(access_token.is_some());
    let access_token = access_token.unwrap();
    assert_eq!(access_token.id, Some(id));
    assert_eq!(access_token.algorithm, "HS256");
}

#[sqlx::test]
async fn test_access_token_repository_read_nonexistent_returns_error(pool: PgPool) {
    let repository = AccessTokenRepository::new(Arc::new(pool));
    let id = Uuid::new_v4();
    let result = repository.read(id).await;
    assert!(result.is_err());
    let error_message = result.unwrap_err().to_string();
    assert_eq!(error_message, "Access token not found");
}

#[sqlx::test(fixtures("../fixtures/access_tokens.sql"))]
async fn test_access_token_repository_update_active_false_succeeds(pool: PgPool) {
    let repository = AccessTokenRepository::new(Arc::new(pool));
    let id = Uuid::parse_str("11111111-1111-1111-1111-111111111111").unwrap();
    let update = AccessTokenUpdatePayload {
        active: Some(false),
    };
    let access_token = repository.update(id, update).await.unwrap();
    assert!(!access_token.active);
}

#[sqlx::test(fixtures("../fixtures/access_tokens.sql"))]
async fn test_access_token_repository_update_active_true_succeeds(pool: PgPool) {
    let repository = AccessTokenRepository::new(Arc::new(pool));
    let id = Uuid::parse_str("22222222-2222-2222-2222-222222222222").unwrap();
    let update = AccessTokenUpdatePayload { active: Some(true) };
    let access_token = repository.update(id, update).await.unwrap();
    assert!(access_token.active);
}

#[sqlx::test(fixtures("../fixtures/access_tokens.sql"))]
async fn test_access_token_repository_delete_existing_succeeds(pool: PgPool) {
    let repository = AccessTokenRepository::new(Arc::new(pool));
    let id = Uuid::parse_str("11111111-1111-1111-1111-111111111111").unwrap();
    let deleted = repository.delete(id).await.unwrap();
    assert!(deleted);
}

#[sqlx::test]
async fn test_access_token_repository_delete_nonexisting_fails(pool: PgPool) {
    let repository = AccessTokenRepository::new(Arc::new(pool));
    let id = Uuid::new_v4();
    let result = repository.delete(id).await;
    assert!(result.is_err());
    let error_message = result.unwrap_err().to_string();
    assert_eq!(error_message, "Access token not found");
}

#[sqlx::test(fixtures("../fixtures/access_tokens.sql"))]
async fn test_access_token_repository_find_with_limit_pagination(pool: PgPool) {
    let repository = AccessTokenRepository::new(Arc::new(pool));
    let filter = AccessTokenFilter::default();
    let sort = None;
    let pagination = Some(Pagination {
        limit: Some(2),
        offset: None,
    });
    let access_tokens = repository.find(filter, sort, pagination).await.unwrap();
    assert_eq!(access_tokens.len(), 2);
}

#[sqlx::test(fixtures("../fixtures/access_tokens.sql"))]
async fn test_access_token_repository_find_with_offset_pagination(pool: PgPool) {
    let repository = AccessTokenRepository::new(Arc::new(pool));
    let filter = AccessTokenFilter::default();
    let sort = None;
    let pagination = Some(Pagination {
        limit: None,
        offset: Some(1),
    });
    let access_tokens = repository.find(filter, sort, pagination).await.unwrap();
    assert_eq!(access_tokens.len(), 3);
}

#[sqlx::test(fixtures("../fixtures/access_tokens.sql"))]
async fn test_access_token_repository_find_with_limit_offset_pagination(pool: PgPool) {
    let repository = AccessTokenRepository::new(Arc::new(pool));
    let filter = AccessTokenFilter::default();
    let sort = None;
    let pagination = Some(Pagination {
        limit: Some(1),
        offset: Some(1),
    });
    let access_tokens = repository.find(filter, sort, pagination).await.unwrap();
    assert_eq!(access_tokens.len(), 1);
}

#[sqlx::test(fixtures("../fixtures/access_tokens.sql"))]
async fn test_access_token_repository_find_with_project_access_id_filter(pool: PgPool) {
    let repository = AccessTokenRepository::new(Arc::new(pool));
    let filter = AccessTokenFilter {
        project_access_id: Some(
            Uuid::parse_str("00000000-0000-0000-0000-000000000101")
                .unwrap()
                .to_string(),
        ),
        ..Default::default()
    };
    let sort = None;
    let pagination = None;
    let access_tokens = repository.find(filter, sort, pagination).await.unwrap();
    assert_eq!(access_tokens.len(), 1);
}

#[sqlx::test(fixtures("../fixtures/access_tokens.sql"))]
async fn test_access_token_repository_find_with_algorithm_filter(pool: PgPool) {
    let repository = AccessTokenRepository::new(Arc::new(pool));
    let filter = AccessTokenFilter {
        algorithm: Some("HS256".to_string()),
        ..Default::default()
    };
    let sort = None;
    let pagination = None;
    let access_tokens = repository.find(filter, sort, pagination).await.unwrap();
    assert_eq!(access_tokens.len(), 2);
}

#[sqlx::test(fixtures("../fixtures/access_tokens.sql"))]
async fn test_access_token_repository_find_with_active_filter(pool: PgPool) {
    let repository = AccessTokenRepository::new(Arc::new(pool));
    let filter = AccessTokenFilter {
        active: Some(false),
        ..Default::default()
    };
    let sort = None;
    let pagination = None;
    let access_tokens = repository.find(filter, sort, pagination).await.unwrap();
    assert_eq!(access_tokens.len(), 4);
}
