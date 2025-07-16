use crate::models::pagination::Pagination;
use crate::models::project_access_scopes::{
    ProjectAccessScopeCreatePayload, ProjectAccessScopeFilter, ProjectAccessScopeResponse,
    ProjectAccessScopeSortOrder, ProjectAccessScopeSortableFields, ProjectAccessScopeUpdatePayload,
};
use crate::models::sort::SortOrder;
use crate::repositories::base::Repository;
use crate::repositories::project_access_scopes_repository::ProjectAccessScopesRepository;
use actix_web::{Error, HttpResponse, web};
use uuid::Uuid;

#[utoipa::path(
    post,
    path = "/project-access-scopes",
    tag = "Project Access Scopes",
    request_body = ProjectAccessScopeCreatePayload,
    responses(
        (status = 201, description = "Project access scope created", body = ProjectAccessScopeResponse),
        (status = 400, description = "Invalid request", body = String),
    ),
)]
pub async fn post(
    repository: web::Data<ProjectAccessScopesRepository>,
    payload: web::Json<ProjectAccessScopeCreatePayload>,
) -> Result<HttpResponse, Error> {
    let project_access_scope = repository
        .create(payload.into_inner())
        .await
        .map_err(actix_web::error::ErrorBadRequest)?;
    Ok(HttpResponse::Created().json(ProjectAccessScopeResponse::from(project_access_scope)))
}

#[utoipa::path(
    get,
    path = "/project-access-scopes/{id}",
    tag = "Project Access Scopes",
    responses(
        (status = 200, description = "Project access scope found", body = ProjectAccessScopeResponse),
        (status = 404, description = "Project access scope not found", body = String),
    ),
    params(
        ("id" = String<uuid::Uuid>, Path, description = "Project Access Scope ID"),
    ),
)]
pub async fn get(
    repository: web::Data<ProjectAccessScopesRepository>,
    id: web::Path<Uuid>,
) -> Result<HttpResponse, Error> {
    let project_access_scope = repository
        .read(id.into_inner())
        .await
        .map_err(actix_web::error::ErrorNotFound)?;
    match project_access_scope {
        Some(scope) => Ok(HttpResponse::Ok().json(ProjectAccessScopeResponse::from(scope))),
        None => Err(actix_web::error::ErrorNotFound(
            "Project access scope not found",
        )),
    }
}

#[utoipa::path(
    patch,
    path = "/project-access-scopes/{id}",
    tag = "Project Access Scopes",
    request_body = ProjectAccessScopeUpdatePayload,
    responses(
        (status = 200, description = "Project access scope updated", body = ProjectAccessScopeResponse),
        (status = 400, description = "Invalid request", body = String),
        (status = 404, description = "Project access scope not found", body = String),
        (status = 409, description = "Project access scope already exists", body = String),
        (status = 500, description = "Internal server error", body = String),
    ),
    params(
        ("id" = String<uuid::Uuid>, Path, description = "Project Access Scope ID"),
    ),
)]
pub async fn patch(
    repository: web::Data<ProjectAccessScopesRepository>,
    id: web::Path<Uuid>,
    payload: web::Json<ProjectAccessScopeUpdatePayload>,
) -> Result<HttpResponse, Error> {
    let project_access_scope = repository
        .update(id.into_inner(), payload.into_inner())
        .await;
    if let Err(error) = project_access_scope {
        let error_message = error.to_string();
        match error_message.as_str() {
            "No changes to update" => return Err(actix_web::error::ErrorBadRequest(error_message)),
            "Project access scope not found" => {
                return Err(actix_web::error::ErrorNotFound(error_message));
            }
            "Project access scope already exists" => {
                return Err(actix_web::error::ErrorConflict(error_message));
            }
            _ => return Err(actix_web::error::ErrorInternalServerError(error_message)),
        }
    }
    Ok(HttpResponse::Ok().json(ProjectAccessScopeResponse::from(
        project_access_scope.unwrap(),
    )))
}

#[utoipa::path(
    delete,
    path = "/project-access-scopes/{id}",
    tag = "Project Access Scopes",
    responses(
        (status = 204, description = "Project access scope deleted", body = ()),
        (status = 404, description = "Project access scope not found", body = String),
    ),
    params(
        ("id" = String<uuid::Uuid>, Path, description = "Project Access Scope ID"),
    )
)]
pub async fn delete(
    repository: web::Data<ProjectAccessScopesRepository>,
    id: web::Path<Uuid>,
) -> Result<HttpResponse, Error> {
    let result = repository.delete(id.into_inner()).await;
    if let Err(error) = result {
        let error_message = error.to_string();
        match error_message.as_str() {
            "Project access scope not found" => {
                return Err(actix_web::error::ErrorNotFound(error_message));
            }
            _ => return Err(actix_web::error::ErrorInternalServerError(error_message)),
        }
    }
    Ok(HttpResponse::NoContent().finish())
}

#[utoipa::path(
    get,
    path = "/project-access-scopes",
    tag = "Project Access Scopes",
    responses(
        (status = 200, description = "Project access scopes found", body = Vec<ProjectAccessScopeResponse>),
    ),
    params(
        ("project_access_id" = Option<String>, Query, description = "Filter by project access id"),
        ("scope_id" = Option<String>, Query, description = "Filter by scope id"),
        ("offset" = Option<u32>, Query, description = "Offset for pagination"),
        ("limit" = Option<u32>, Query, description = "Number of items per page"),
    )
)]
pub async fn list(
    repository: web::Data<ProjectAccessScopesRepository>,
    filter: web::Query<ProjectAccessScopeFilter>,
    pagination: web::Query<Pagination>,
) -> Result<HttpResponse, Error> {
    let sort = vec![ProjectAccessScopeSortOrder::new(
        ProjectAccessScopeSortableFields::Id,
        SortOrder::Asc,
    )];
    let project_access_scopes = repository
        .find(
            filter.into_inner(),
            Some(sort),
            Some(pagination.into_inner()),
        )
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;
    let responses: Vec<ProjectAccessScopeResponse> = project_access_scopes
        .into_iter()
        .map(ProjectAccessScopeResponse::from)
        .collect();
    Ok(HttpResponse::Ok().json(responses))
}

pub fn configure_routes(config: &mut actix_web::web::ServiceConfig) {
    config.service(
        web::scope("/project-access-scopes")
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
