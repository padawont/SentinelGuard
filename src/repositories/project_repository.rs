use std::sync::Arc;

use crate::models::project::ProjectCreatePayload;
use crate::models::{
    pagination::Pagination,
    project::{Project, ProjectFilter, ProjectSortOrder, ProjectUpdatePayload},
};
use crate::repositories::base::Repository;
use anyhow::Error;
use async_trait::async_trait;
use chrono::Utc;
use sqlx::QueryBuilder;
use sqlx::Row;
use uuid::Uuid;

use sqlx;

#[derive(Clone)]
pub struct ProjectRepository {
    pub pool: Arc<sqlx::postgres::PgPool>,
}

impl ProjectRepository {
    pub fn new(pool: Arc<sqlx::postgres::PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl Repository<Project> for ProjectRepository {
    type CreatePayload = ProjectCreatePayload;
    type UpdatePayload = ProjectUpdatePayload;
    type Filter = ProjectFilter;
    type Sort = ProjectSortOrder;

    async fn create(&self, item: Self::CreatePayload) -> Result<Project, Error> {
        let project = Project {
            id: None,
            name: item.name,
            description: item.description,
            enabled: item.enabled,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let created_project = sqlx::query_as!(
            Project,
            "INSERT INTO projects (name, description, enabled) VALUES ($1, $2, $3) RETURNING id, name, description, enabled, created_at, updated_at",
            project.name,
            project.description,
            project.enabled,
        )
        .fetch_one(&*self.pool)
        .await
        .map_err(<sqlx::Error as Into<Error>>::into);

        if created_project.is_err() {
            return Err(Error::msg(
                "Failed to create project: ".to_owned() + &created_project.unwrap_err().to_string(),
            ));
        }

        Ok(created_project.unwrap())
    }

    async fn read(&self, id: Uuid) -> Result<Option<Project>, Error> {
        let project = sqlx::query_as!(
            Project,
            "SELECT id, name, description, enabled, created_at, updated_at FROM projects WHERE id = $1",
            id
        )
        .fetch_optional(&*self.pool)
        .await
        .map_err(<sqlx::Error as Into<Error>>::into)?;

        if project.is_none() {
            return Err(Error::msg("Project not found"));
        }

        Ok(project)
    }

    async fn update(&self, id: Uuid, update: Self::UpdatePayload) -> Result<Project, Error> {
        let mut changes = Vec::new();

        if let Some(name) = update.name {
            changes.push(("name", name));
        }

        if let Some(description) = update.description {
            changes.push(("description", description));
        }

        if let Some(enabled) = update.enabled {
            match enabled {
                true => changes.push(("enabled = true", "".to_string())),
                false => changes.push(("enabled = false", "".to_string())),
            }
        }

        if changes.is_empty() {
            return Err(Error::msg("No changes to update"));
        }

        let mut query = QueryBuilder::new("UPDATE projects SET ");

        let mut separated = query.separated(", ");
        for (field, value) in changes {
            if value.is_empty() {
                separated.push(field);
            } else {
                separated
                    .push(format!("{} = ", field))
                    .push_bind_unseparated(value);
            }
        }

        query.push(", updated_at = ").push_bind(Utc::now());

        query.push(" WHERE id = ").push_bind(id);

        query.push(" RETURNING id, name, description, enabled, created_at, updated_at");

        let result = query
            .build()
            .fetch_one(&*self.pool)
            .await
            .map(|row| Project {
                id: row.get("id"),
                name: row.get("name"),
                description: row.get("description"),
                enabled: row.get("enabled"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            });

        match result {
            Ok(project) => Ok(project),
            Err(error) => match error {
                sqlx::Error::RowNotFound => Err(Error::msg("Project not found")),
                sqlx::Error::Database(e) => {
                    let error_message = e.message();

                    match error_message {
                        s if s.contains("unique constraint") || s.contains("duplicate key") => {
                            Err(Error::msg("Project name already exists"))
                        }
                        _ => Err(Error::msg("No changes were made")),
                    }
                }
                _ => Err(error.into()),
            },
        }
    }

    async fn delete(&self, id: Uuid) -> Result<bool, Error> {
        let result = sqlx::query!("DELETE FROM projects WHERE id = $1", id)
            .execute(&*self.pool)
            .await;

        match result {
            Ok(result) => Ok(result.rows_affected() == 1),
            Err(error) => Err(error.into()),
        }
    }

    async fn find(
        &self,
        filter: Self::Filter,
        sort: Option<Vec<Self::Sort>>,
        pagination: Option<Pagination>,
    ) -> Result<Vec<Project>, Error> {
        let mut query = QueryBuilder::new(
            "SELECT id, name, description, enabled, created_at, updated_at FROM projects ",
        );

        let mut conditions_list = Vec::new();

        if let Some(name) = filter.name {
            conditions_list.push(("name ILIKE ", format!("%{}%", name)));
        }

        if let Some(description) = filter.description {
            conditions_list.push(("description ILIKE ", format!("%{}%", description)));
        }

        if let Some(enabled) = filter.enabled {
            match enabled {
                true => conditions_list.push(("enabled = true", "".to_string())),
                false => conditions_list.push(("enabled = false", "".to_string())),
            }
        }

        if !conditions_list.is_empty() {
            query.push("WHERE ");
            let mut conditions = query.separated(" AND ");
            for (condition, value) in conditions_list {
                if value.is_empty() {
                    conditions.push(condition);
                } else {
                    conditions.push(condition).push_bind_unseparated(value);
                }
            }
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

        let projects: Vec<Project> = query
            .build()
            .fetch_all(&*self.pool)
            .await
            .map_err(<sqlx::Error as Into<Error>>::into)?
            .into_iter()
            .map(|row| Project {
                id: row.get("id"),
                name: row.get("name"),
                description: row.get("description"),
                enabled: row.get("enabled"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            })
            .collect();

        Ok(projects)
    }
}
