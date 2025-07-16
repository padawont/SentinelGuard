use std::sync::Arc;

use anyhow::Error;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{QueryBuilder, Row};
use uuid::Uuid;

use crate::{
    models::{
        access_token::{
            AccessToken, AccessTokenCreatePayloadWithAccessToken, AccessTokenFilter,
            AccessTokenSortOrder, AccessTokenUpdatePayload,
        },
        pagination::Pagination,
    },
    repositories::base::Repository,
};

#[derive(Clone)]
pub struct AccessTokenRepository {
    pub pool: Arc<sqlx::postgres::PgPool>,
}

impl AccessTokenRepository {
    pub fn new(pool: Arc<sqlx::postgres::PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl Repository<AccessToken> for AccessTokenRepository {
    type CreatePayload = AccessTokenCreatePayloadWithAccessToken;
    type UpdatePayload = AccessTokenUpdatePayload;
    type Filter = AccessTokenFilter;
    type Sort = AccessTokenSortOrder;

    async fn create(&self, item: Self::CreatePayload) -> Result<AccessToken, Error> {
        let access_token = AccessToken {
            id: None,
            project_access_id: item.project_access_id.parse().unwrap(),
            algorithm: item.algorithm,
            token: item.access_token,
            active: true,
            expires_at: DateTime::parse_from_rfc3339(&item.expires_at)
                .unwrap()
                .with_timezone(&Utc),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let created_access_token = sqlx::query_as!(
            AccessToken,
            "INSERT INTO access_tokens (project_access_id, algorithm, token, expires_at, active) VALUES ($1, $2, $3, $4, $5) RETURNING id, project_access_id, algorithm, token, expires_at, active, created_at, updated_at",
            access_token.project_access_id,
            access_token.algorithm,
            access_token.token,
            access_token.expires_at,
            access_token.active,
        )
        .fetch_one(&*self.pool)
        .await;

        match created_access_token {
            Ok(access_token) => Ok(access_token),
            Err(error) => Err(error.into()),
        }
    }

    async fn read(&self, id: Uuid) -> Result<Option<AccessToken>, Error> {
        let access_token = sqlx::query_as!(
            AccessToken,
            "SELECT id, project_access_id, algorithm, token, expires_at, active, created_at, updated_at FROM access_tokens WHERE id = $1 LIMIT 1",
            id,
        )
        .fetch_optional(&*self.pool)
        .await
        .map_err(<sqlx::Error as Into<Error>>::into)?;

        if access_token.is_none() {
            return Err(Error::msg("Access token not found"));
        }

        Ok(access_token)
    }

    async fn update(&self, id: Uuid, update: Self::UpdatePayload) -> Result<AccessToken, Error> {
        let mut changes = Vec::new();

        if let Some(active) = update.active {
            match active {
                true => changes.push(("active = true", "".to_string())),
                false => changes.push(("active = false", "".to_string())),
            }
        }

        if changes.is_empty() {
            return Err(Error::msg("No changes to update"));
        }

        let mut query = QueryBuilder::new("UPDATE access_tokens SET ");
        let mut separated = query.separated(", ");
        for (field, value) in changes {
            if value.is_empty() {
                separated.push(field.to_string());
            } else {
                separated
                    .push(format!("{} = ", field))
                    .push_bind_unseparated(value);
            }
        }
        query.push(", updated_at = ").push_bind(Utc::now());
        query.push(" WHERE id = ").push_bind(id);
        query.push(" RETURNING id, project_access_id, algorithm, token, expires_at, active, created_at, updated_at");

        let result = query
            .build()
            .fetch_one(&*self.pool)
            .await
            .map(|row| AccessToken {
                id: row.get("id"),
                project_access_id: row.get("project_access_id"),
                algorithm: row.get("algorithm"),
                token: row.get("token"),
                expires_at: row.get("expires_at"),
                active: row.get("active"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            });

        match result {
            Ok(access_token) => Ok(access_token),
            Err(error) => Err(error.into()),
        }
    }

    async fn delete(&self, id: Uuid) -> Result<bool, Error> {
        let deleted = sqlx::query!("DELETE FROM access_tokens WHERE id = $1 RETURNING id", id)
            .fetch_optional(&*self.pool)
            .await
            .map_err(<sqlx::Error as Into<Error>>::into)?;

        if deleted.is_none() {
            return Err(Error::msg("Access token not found"));
        }

        Ok(true)
    }

    async fn find(
        &self,
        filter: Self::Filter,
        sort: Option<Vec<Self::Sort>>,
        pagination: Option<Pagination>,
    ) -> Result<Vec<AccessToken>, Error> {
        let mut query = QueryBuilder::new(
            "SELECT id, project_access_id, algorithm, token, active, expires_at, created_at, updated_at FROM access_tokens ",
        );

        let mut conditions_list: Vec<(&str, String)> = Vec::new();

        if let Some(algorithm) = &filter.algorithm {
            conditions_list.push(("algorithm = ", algorithm.clone()));
        }

        if !&conditions_list.is_empty() {
            query.push("WHERE ");
            let mut conditions = query.separated(" AND ");
            for (condition, value) in &conditions_list {
                conditions.push(condition).push_bind_unseparated(value);
            }
        }

        if let Some(project_access_id) = &filter.project_access_id {
            if conditions_list.is_empty() {
                query.push("WHERE ");
            } else {
                query.push(" AND ");
            }
            let project_access_id = uuid::Uuid::parse_str(project_access_id).unwrap();

            query
                .push(" project_access_id = ")
                .push_bind(project_access_id);
        }

        if let Some(sort) = sort {
            query.push(" ORDER BY ");
            let mut order_by = query.separated(", ");
            for sort in sort {
                let field = String::from(sort.field);
                order_by.push(format!("{} {}", field, sort.order));
            }
        }

        if let Some(pagination) = pagination {
            query
                .push(" LIMIT ")
                .push_bind(pagination.limit.unwrap_or(10));
            query
                .push(" OFFSET ")
                .push_bind(pagination.offset.unwrap_or(0));
        }

        let access_tokens = query
            .build()
            .fetch_all(&*self.pool)
            .await
            .map_err(<sqlx::Error as Into<Error>>::into)?
            .into_iter()
            .map(|row| AccessToken {
                id: row.get("id"),
                project_access_id: row.get("project_access_id"),
                algorithm: row.get("algorithm"),
                token: row.get("token"),
                expires_at: row.get("expires_at"),
                active: row.get("active"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            })
            .collect();

        Ok(access_tokens)
    }
}
