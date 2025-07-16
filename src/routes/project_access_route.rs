use crate::models::pagination::Pagination;
use crate::models::project_access::ProjectAccessCreatePayload;
use crate::models::project_access::{
    ProjectAccessFilter, ProjectAccessResponse, ProjectAccessSortOrder,
    ProjectAccessSortableFields, ProjectAccessUpdatePayload,
};
use crate::models::sort::SortOrder;
use crate::repositories::base::Repository;
use crate::repositories::project_access_repository::ProjectAccessRepository;
use actix_web::{Error, HttpResponse, web};
use uuid::Uuid;

#[utoipa::path(
    post,
    path = "/projects/{project_id}/access",
    tag = "Projects",
    request_body = ProjectAccessCreatePayload,
    responses(
        (status = 201, description = "Project access created", body = ProjectAccessResponse),
        (status = 400, description = "Invalid request", body = String),
    ),
)]
pub async fn post(
    repository: web::Data<ProjectAccessRepository>,
    payload: web::Json<ProjectAccessCreatePayload>,
) -> Result<HttpResponse, Error> {
    let project_access = repository
        .create(payload.into_inner())
        .await
        .map_err(actix_web::error::ErrorBadRequest)?;
    Ok(HttpResponse::Created().json(project_access))
}

#[utoipa::path(
    get,
    path = "/projects/{project_id}/access/{id}",
    tag = "Projects",
    responses(
        (status = 200, description = "Project access retrieved", body = ProjectAccessResponse), 
        (status = 400, description = "Invalid request", body = String),
    ),
)]
pub async fn get(
    repository: web::Data<ProjectAccessRepository>,
    id: web::Path<Uuid>,
) -> Result<HttpResponse, Error> {
    let project_access = repository
        .read(id.into_inner())
        .await
        .map_err(actix_web::error::ErrorBadRequest)?;
    Ok(HttpResponse::Ok().json(project_access))
}

#[utoipa::path(
    patch,
    path = "/projects/{project_id}/access/{id}",
    tag = "Projects",
    request_body = ProjectAccessUpdatePayload,
    responses(
        (status = 200, description = "Project access updated", body = ProjectAccessResponse),
        (status = 400, description = "Invalid request", body = String),
    ),
)]
pub async fn patch(
    repository: web::Data<ProjectAccessRepository>,
    id: web::Path<Uuid>,
    payload: web::Json<ProjectAccessUpdatePayload>,
) -> Result<HttpResponse, Error> {
    let project_access = repository
        .update(id.into_inner(), payload.into_inner())
        .await
        .map_err(actix_web::error::ErrorBadRequest)?;
    Ok(HttpResponse::Ok().json(project_access))
}

#[utoipa::path(
    delete,
    path = "/projects/{project_id}/access/{id}",
    tag = "Projects",
    responses(
        (status = 200, description = "Project access deleted", body = ProjectAccessResponse),
        (status = 400, description = "Invalid request", body = String),
    ),
)]
pub async fn delete(
    repository: web::Data<ProjectAccessRepository>,
    id: web::Path<Uuid>,
) -> Result<HttpResponse, Error> {
    let project_access = repository
        .delete(id.into_inner())
        .await
        .map_err(actix_web::error::ErrorBadRequest)?;
    Ok(HttpResponse::Ok().json(project_access))
}

#[utoipa::path(
    get,
    path = "/projects/{project_id}/access",
    tag = "Projects",
    responses(
            (status = 200, description = "Project Access updated", body = ProjectAccessResponse),
            (status = 400, description = "Invalid request", body = String),
            (status = 404, description = "Project Access not found", body = String),
            (status = 409, description = "Project Id, Service Account Id and Environment Id combination already exists", body = String),
            (status = 500, description = "Internal server error", body = String),
    ),
)]
pub async fn list(
    repository: web::Data<ProjectAccessRepository>,
    filter: web::Query<ProjectAccessFilter>,
    pagination: web::Query<Pagination>,
) -> Result<HttpResponse, Error> {
    let sort = vec![ProjectAccessSortOrder {
        field: ProjectAccessSortableFields::Id,
        order: SortOrder::Desc,
    }];

    let project_access = repository
        .find(
            filter.into_inner(),
            Some(sort),
            Some(pagination.into_inner()),
        )
        .await
        .map_err(actix_web::error::ErrorBadRequest)?;
    Ok(HttpResponse::Ok().json(project_access))
}

pub fn configure_routes(config: &mut actix_web::web::ServiceConfig) {
    config.service(
        web::scope("/project-access")
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
