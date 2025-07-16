use crate::models::pagination::Pagination;
use crate::models::project::{ProjectCreatePayload, ProjectUpdatePayload};
use crate::models::project::{
    ProjectFilter, ProjectResponse, ProjectSortOrder, ProjectSortableFields,
};
use crate::models::sort::SortOrder;
use crate::repositories::base::Repository;
use crate::repositories::project_repository::ProjectRepository;
use actix_web::{Error, HttpResponse, web};

#[utoipa::path(
    post,
    path = "/projects",
    tag = "Projects",
    request_body = ProjectCreatePayload,
    responses(
        (status = 201, description = "Project created", body = ProjectResponse),
        (status = 400, description = "Invalid request", body = String),
    ),
)]
pub async fn post(
    repository: web::Data<ProjectRepository>,
    payload: web::Json<ProjectCreatePayload>,
) -> Result<HttpResponse, Error> {
    let project = repository
        .create(payload.into_inner())
        .await
        .map_err(actix_web::error::ErrorBadRequest)?;
    Ok(HttpResponse::Created().json(project))
}

#[utoipa::path(
    get,
    path = "/projects/{id}",
    tag = "Projects",
    responses(
        (status = 200, description = "Project found", body = ProjectResponse),
        (status = 404, description = "Project not found", body = String),
    ),
    params(
        ("id" = String<uuid::Uuid>, Path, description = "Project ID"),
    ),
)]
pub async fn get(
    repository: web::Data<ProjectRepository>,
    id: web::Path<uuid::Uuid>,
) -> Result<HttpResponse, Error> {
    let project = repository
        .read(id.into_inner())
        .await
        .map_err(actix_web::error::ErrorNotFound)?;
    Ok(HttpResponse::Ok().json(project))
}

#[utoipa::path(
    patch,
    path = "/projects/{id}",
    tag = "Projects",
    responses(
        (status = 200, description = "Project updated", body = ProjectResponse),
        (status = 400, description = "Invalid request", body = String),
        (status = 404, description = "Project not found", body = String),
        (status = 409, description = "Project name already exists", body = String),
        (status = 500, description = "Internal server error", body = String),
    ),
    params(
        ("id" = String<uuid::Uuid>, Path, description = "Project ID"),
    ),
)]
pub async fn patch(
    repository: web::Data<ProjectRepository>,
    id: web::Path<uuid::Uuid>,
    payload: web::Json<ProjectUpdatePayload>,
) -> Result<HttpResponse, Error> {
    let project = repository
        .update(id.into_inner(), payload.into_inner())
        .await;
    if project.is_err() {
        let error_message = project.unwrap_err().to_string();
        match error_message.as_str() {
            "No changes to update" => return Err(actix_web::error::ErrorBadRequest(error_message)),
            "Project not found" => return Err(actix_web::error::ErrorNotFound(error_message)),
            "Project name already exists" => {
                return Err(actix_web::error::ErrorConflict(error_message));
            }
            _ => return Err(actix_web::error::ErrorInternalServerError(error_message)),
        }
    }
    Ok(HttpResponse::Ok().json(project.unwrap()))
}

#[utoipa::path(
    delete,
    path = "/projects/{id}",
    tag = "Projects",
    responses(
        (status = 204, description = "Project deleted", body = ()),
        (status = 404, description = "Project not found", body = String),
    ),
    params(
        ("id" = String<uuid::Uuid>, Path, description = "Project ID"),
    )
)]
pub async fn delete(
    repository: web::Data<ProjectRepository>,
    id: web::Path<uuid::Uuid>,
) -> Result<HttpResponse, Error> {
    let result = repository.delete(id.into_inner()).await;
    match result {
        Ok(true) => Ok(HttpResponse::NoContent().finish()),
        Ok(false) => Err(actix_web::error::ErrorNotFound("Project not found")),
        Err(error) => Err(actix_web::error::ErrorInternalServerError(error)),
    }
}

#[utoipa::path(
    get,
    path = "/projects",
    tag = "Projects",
    responses(
        (status = 200, description = "Projects found", body = Vec<ProjectResponse>),
    ),
    params(
        ("name" = Option<String>, Query, description = "Filter projects by name"),
        ("description" = Option<String>, Query, description = "Filter projects by description"),
        ("enabled" = Option<bool>, Query, description = "Filter projects by enabled"),
        ("offset" = Option<u32>, Query, description = "Offset for pagination"),
        ("limit" = Option<u32>, Query, description = "Number of items per page"),
    )
)]
pub async fn list(
    repository: web::Data<ProjectRepository>,
    filter: web::Query<ProjectFilter>,
    pagination: web::Query<Pagination>,
) -> Result<HttpResponse, Error> {
    let sort = vec![ProjectSortOrder::new(
        ProjectSortableFields::Id,
        SortOrder::Asc,
    )];
    let projects = repository
        .find(
            filter.into_inner(),
            Some(sort),
            Some(pagination.into_inner()),
        )
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().json(projects))
}

pub fn configure_routes(config: &mut actix_web::web::ServiceConfig) {
    config.service(
        web::scope("/projects")
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
