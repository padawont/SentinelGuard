use actix_web::dev::{ServiceFactory, ServiceRequest, ServiceResponse};
use actix_web::{App, web};
use sqlx::postgres::PgPool;
use std::sync::Arc;

use crate::repositories::{
    environment_key_repository::EnvironmentKeyRepository,
    environment_repository::EnvironmentRepository,
    project_access_repository::ProjectAccessRepository,
    project_access_scopes_repository::ProjectAccessScopesRepository,
    project_repository::ProjectRepository, project_scope_repository::ProjectScopeRepository,
    service_account_repository::ServiceAccountRepository,
};

pub fn register_repositories<T>(app: App<T>, pool: Arc<PgPool>) -> App<T>
where
    T: ServiceFactory<
            ServiceRequest,
            Config = (),
            Response = ServiceResponse,
            Error = actix_web::Error,
            InitError = (),
        >,
{
    app.app_data(web::Data::new(ProjectRepository::new(pool.clone())))
        .app_data(web::Data::new(ServiceAccountRepository::new(pool.clone())))
        .app_data(web::Data::new(ProjectScopeRepository::new(pool.clone())))
        .app_data(web::Data::new(EnvironmentRepository::new(pool.clone())))
        .app_data(web::Data::new(ProjectAccessRepository::new(pool.clone())))
        .app_data(web::Data::new(ProjectAccessScopesRepository::new(
            pool.clone(),
        )))
        .app_data(web::Data::new(EnvironmentKeyRepository::new(pool.clone())))
}
