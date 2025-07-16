use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::models::sort::SortOrder;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct AccessToken {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Uuid>,
    pub project_access_id: Uuid,
    pub algorithm: String,
    pub token: String,
    pub expires_at: DateTime<Utc>,
    pub active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Default, ToSchema)]
pub struct AccessTokenResponse {
    #[schema(example = "123e4567-e89b-12d3-a456-426614174000")]
    pub id: String,
    #[schema(example = "123e4567-e89b-12d3-a456-426614174000")]
    pub project_access_id: String,
    #[schema(example = "HS256")]
    pub algorithm: String,
    #[schema(example = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...")]
    pub token: String,
    #[schema(example = "2025-07-01T00:00:00.000Z")]
    pub expires_at: String,
    #[schema(example = "true")]
    pub active: bool,
    #[schema(example = "2025-06-25T12:00:00.000Z")]
    pub created_at: String,
    #[schema(example = "2025-06-25T12:00:00.000Z")]
    pub updated_at: String,
}

impl From<AccessToken> for AccessTokenResponse {
    fn from(value: AccessToken) -> Self {
        Self {
            id: value.id.unwrap().to_string(),
            project_access_id: value.project_access_id.to_string(),
            algorithm: value.algorithm,
            token: value.token,
            expires_at: value.expires_at.to_string(),
            active: value.active,
            created_at: value.created_at.to_string(),
            updated_at: value.updated_at.to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct AccessTokenCreatePayload {
    pub project_access_id: String,
    pub algorithm: String,
    pub expires_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct AccessTokenCreatePayloadWithAccessToken {
    pub project_access_id: String,
    pub algorithm: String,
    pub expires_at: String,
    pub access_token: String,
}

impl From<AccessTokenCreatePayloadWithAccessToken> for AccessTokenCreatePayload {
    fn from(value: AccessTokenCreatePayloadWithAccessToken) -> Self {
        Self {
            project_access_id: value.project_access_id,
            algorithm: value.algorithm,
            expires_at: value.expires_at,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct AccessTokenUpdatePayload {
    pub active: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Default, ToSchema)]
pub struct AccessTokenFilter {
    pub project_access_id: Option<String>,
    pub algorithm: Option<String>,
    pub active: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum AccessTokenSortableFields {
    Id,
    ProjectAccessId,
    Algorithm,
    ExpiresAt,
    CreatedAt,
    UpdatedAt,
}

impl From<AccessTokenSortableFields> for String {
    fn from(value: AccessTokenSortableFields) -> Self {
        match value {
            AccessTokenSortableFields::Id => "id".to_string(),
            AccessTokenSortableFields::ProjectAccessId => "project_access_id".to_string(),
            AccessTokenSortableFields::Algorithm => "algorithm".to_string(),
            AccessTokenSortableFields::ExpiresAt => "expires_at".to_string(),
            AccessTokenSortableFields::CreatedAt => "created_at".to_string(),
            AccessTokenSortableFields::UpdatedAt => "updated_at".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AccessTokenSortOrder {
    pub field: AccessTokenSortableFields,
    pub order: SortOrder,
}

impl AccessTokenSortOrder {
    pub fn new(field: AccessTokenSortableFields, order: SortOrder) -> Self {
        Self { field, order }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};
    use uuid::Uuid;

    #[test]
    fn test_access_token_default() {
        let access_token = AccessToken::default();
        assert!(access_token.id.is_none());
        assert_eq!(access_token.project_access_id, Uuid::nil());
        assert_eq!(access_token.algorithm, "");
        assert_eq!(access_token.token, "");
    }

    #[test]
    fn test_access_token_filter_default() {
        let filter = AccessTokenFilter::default();
        assert!(filter.project_access_id.is_none());
        assert!(filter.algorithm.is_none());
        assert!(filter.active.is_none());
    }

    #[test]
    fn test_access_token_sortable_fields_to_string() {
        assert_eq!(String::from(AccessTokenSortableFields::Id), "id");
        assert_eq!(
            String::from(AccessTokenSortableFields::ProjectAccessId),
            "project_access_id"
        );
        assert_eq!(
            String::from(AccessTokenSortableFields::Algorithm),
            "algorithm"
        );
        assert_eq!(
            String::from(AccessTokenSortableFields::ExpiresAt),
            "expires_at"
        );
        assert_eq!(
            String::from(AccessTokenSortableFields::CreatedAt),
            "created_at"
        );
        assert_eq!(
            String::from(AccessTokenSortableFields::UpdatedAt),
            "updated_at"
        );
    }

    #[test]
    fn test_access_token_sort_order_new() {
        let sort = AccessTokenSortOrder::new(AccessTokenSortableFields::Algorithm, SortOrder::Asc);
        assert!(matches!(sort.field, AccessTokenSortableFields::Algorithm));
        assert!(matches!(sort.order, SortOrder::Asc));
    }

    #[test]
    fn test_access_token_to_response() {
        let id = Uuid::new_v4();
        let project_access_id = Uuid::new_v4();
        let now = Utc.with_ymd_and_hms(2025, 6, 25, 12, 0, 0).unwrap();
        let expires = Utc.with_ymd_and_hms(2025, 7, 1, 0, 0, 0).unwrap();

        let access_token = AccessToken {
            id: Some(id),
            project_access_id,
            algorithm: "HS256".to_string(),
            token: "sometoken".to_string(),
            expires_at: expires,
            active: true,
            created_at: now,
            updated_at: now,
        };

        let response = AccessTokenResponse::from(access_token);
        assert_eq!(response.id, id.to_string());
        assert_eq!(response.project_access_id, project_access_id.to_string());
        assert_eq!(response.algorithm, "HS256");
        assert_eq!(response.token, "sometoken");
        assert_eq!(response.expires_at, expires.to_string());
        assert!(response.active);
        assert_eq!(response.created_at, now.to_string());
        assert_eq!(response.updated_at, now.to_string());
    }
}
