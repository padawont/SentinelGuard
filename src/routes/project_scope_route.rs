use crate::models::pagination::Pagination;
use crate::models::project_scope::{
    ProjectScopeCreatePayload, ProjectScopeFilter, ProjectScopeResponse, ProjectScopeSortOrder,
    ProjectScopeSortableFields, ProjectScopeUpdatePayload,
};
use crate::models::sort::SortOrder;
use crate::repositories::base::Repository;
use crate::repositories::project_scope_repository::ProjectScopeRepository;
use actix_web::{Error, HttpResponse, web};

#[utoipa::path(
    post,
    path = "/project-scopes",
    tag = "Project Scopes",
    request_body = ProjectScopeCreatePayload,
    responses(
        (status = 201, description = "Project scope created", body = ProjectScopeResponse),
        (status = 400, description = "Invalid request", body = String),
    ),
)]
pub async fn post(
    repository: web::Data<ProjectScopeRepository>,
    payload: web::Json<ProjectScopeCreatePayload>,
) -> Result<HttpResponse, Error> {
    let project_scope = repository
        .create(payload.into_inner())
        .await
        .map_err(actix_web::error::ErrorBadRequest)?;
    Ok(HttpResponse::Created().json(ProjectScopeResponse::from(project_scope)))
}

#[utoipa::path(
    get,
    path = "/project-scopes/{id}",
    tag = "Project Scopes",
    responses(
        (status = 200, description = "Project scope found", body = ProjectScopeResponse),
        (status = 404, description = "Project scope not found", body = String),
    ),
    params(
        ("id" = String<uuid::Uuid>, Path, description = "Project Scope ID"),
    ),
)]
pub async fn get(
    repository: web::Data<ProjectScopeRepository>,
    id: web::Path<uuid::Uuid>,
) -> Result<HttpResponse, Error> {
    let project_scope = repository
        .read(id.into_inner())
        .await
        .map_err(actix_web::error::ErrorNotFound)?;

    Ok(HttpResponse::Ok().json(project_scope))
}

#[utoipa::path(
    patch,
    path = "/project-scopes/{id}",
    tag = "Project Scopes",
    responses(
        (status = 200, description = "Project scope updated", body = ProjectScopeResponse),
        (status = 400, description = "Invalid request", body = String),
        (status = 404, description = "Project scope not found", body = String),
        (status = 409, description = "Project scope already exists", body = String),
        (status = 500, description = "Internal server error", body = String),
    ),
    params(
        ("id" = String<uuid::Uuid>, Path, description = "Project Scope ID"),
    ),
)]
pub async fn patch(
    repository: web::Data<ProjectScopeRepository>,
    id: web::Path<uuid::Uuid>,
    payload: web::Json<ProjectScopeUpdatePayload>,
) -> Result<HttpResponse, Error> {
    let project_scope = repository
        .update(id.into_inner(), payload.into_inner())
        .await;

    if project_scope.is_err() {
        let error_message = project_scope.unwrap_err().to_string();
        match error_message.as_str() {
            "No changes to update" => return Err(actix_web::error::ErrorBadRequest(error_message)),
            "Project scope not found" => {
                return Err(actix_web::error::ErrorNotFound(error_message));
            }
            "Project Id, scope combination already exists" => {
                return Err(actix_web::error::ErrorConflict(error_message));
            }
            _ => return Err(actix_web::error::ErrorInternalServerError(error_message)),
        }
    }

    Ok(HttpResponse::Ok().json(ProjectScopeResponse::from(project_scope.unwrap())))
}

#[utoipa::path(
    delete,
    path = "/project-scopes/{id}",
    tag = "Project Scopes",
    responses(
        (status = 204, description = "Project scope deleted", body = ()),
        (status = 404, description = "Project scope not found", body = String),
    ),
    params(
        ("id" = String<uuid::Uuid>, Path, description = "Project scope ID"),
    )
)]
pub async fn delete(
    repository: web::Data<ProjectScopeRepository>,
    id: web::Path<uuid::Uuid>,
) -> Result<HttpResponse, Error> {
    let result = repository.delete(id.into_inner()).await;

    if result.is_err() {
        let error_message = result.unwrap_err().to_string();
        match error_message.as_str() {
            "Project scope not found" => {
                return Err(actix_web::error::ErrorNotFound(error_message));
            }
            _ => return Err(actix_web::error::ErrorInternalServerError(error_message)),
        }
    }

    Ok(HttpResponse::NoContent().finish())
}

#[utoipa::path(
    get,
    path = "/project-scopes",
    tag = "Project Scopes",
    responses(
        (status = 200, description = "Project scopes found", body = Vec<ProjectScopeResponse>),
    ),
    params(
        ("project_id" = Option<String>, Query, description = "Filter project scopes by project ID"),
        ("scope" = Option<String>, Query, description = "Filter project scopes by scope"),
        ("description" = Option<String>, Query, description = "Filter project scopes by description"),
        ("enabled" = Option<bool>, Query, description = "Filter project scopes by enabled status"),
        ("offset" = Option<u32>, Query, description = "Offset for pagination"),
        ("limit" = Option<u32>, Query, description = "Number of items per page"),
    )
)]
pub async fn list(
    repository: web::Data<ProjectScopeRepository>,
    filter: web::Query<ProjectScopeFilter>,
    pagination: web::Query<Pagination>,
) -> Result<HttpResponse, Error> {
    let sort = vec![ProjectScopeSortOrder::new(
        ProjectScopeSortableFields::Id,
        SortOrder::Asc,
    )];
    let project_scopes = repository
        .find(
            filter.into_inner(),
            Some(sort),
            Some(pagination.into_inner()),
        )
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    let responses: Vec<ProjectScopeResponse> = project_scopes
        .into_iter()
        .map(ProjectScopeResponse::from)
        .collect();

    Ok(HttpResponse::Ok().json(responses))
}

pub fn configure_routes(config: &mut actix_web::web::ServiceConfig) {
    config.service(
        web::scope("/project-scopes")
            .service(
                actix_web::web::resource("")
                    .route(actix_web::web::post().to(post))
                    .route(actix_web::web::get().to(list)),
            )
            .service(
                actix_web::web::resource("/{id}")
                    .route(actix_web::web::get().to(get))
                    .route(actix_web::web::patch().to(patch))
                    .route(actix_web::web::delete().to(delete)),
            ),
    );
}
