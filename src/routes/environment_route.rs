use crate::models::environment::{
    EnvironmentCreatePayload, EnvironmentFilter, EnvironmentResponse, EnvironmentSortOrder,
    EnvironmentSortableFields, EnvironmentUpdatePayload,
};
use crate::models::pagination::Pagination;
use crate::models::sort::SortOrder;
use crate::repositories::base::Repository;
use crate::repositories::environment_repository::EnvironmentRepository;
use actix_web::{Error, HttpResponse, web};

#[utoipa::path(
    post,
    path = "/environments",
    tag = "Environments",
    request_body = EnvironmentCreatePayload,
    responses(
        (status = 201, description = "Environment created", body = EnvironmentResponse),
        (status = 400, description = "Invalid request", body = String),
    ),
)]
pub async fn post(
    repository: web::Data<EnvironmentRepository>,
    payload: web::Json<EnvironmentCreatePayload>,
) -> Result<HttpResponse, Error> {
    let environment = repository
        .create(payload.into_inner())
        .await
        .map_err(actix_web::error::ErrorBadRequest)?;
    Ok(HttpResponse::Created().json(EnvironmentResponse::from(environment)))
}

#[utoipa::path(
    get,
    path = "/environments/{id}",
    tag = "Environments",
    responses(
        (status = 200, description = "Environment found", body = EnvironmentResponse),
        (status = 404, description = "Environment not found", body = String),
    ),
    params(
        ("id" = String<uuid::Uuid>, Path, description = "Environment ID"),
    ),
)]
pub async fn get(
    repository: web::Data<EnvironmentRepository>,
    id: web::Path<uuid::Uuid>,
) -> Result<HttpResponse, Error> {
    let environment = repository
        .read(id.into_inner())
        .await
        .map_err(actix_web::error::ErrorNotFound)?;

    Ok(HttpResponse::Ok().json(environment))
}

#[utoipa::path(
    patch,
    path = "/environments/{id}",
    tag = "Environments",
    responses(
        (status = 200, description = "Environment updated", body = EnvironmentResponse),
        (status = 400, description = "Invalid request", body = String),
        (status = 404, description = "Environment not found", body = String),
        (status = 409, description = "Environment already exists", body = String),
        (status = 500, description = "Internal server error", body = String),
    ),
    params(
        ("id" = String<uuid::Uuid>, Path, description = "Environment ID"),
    ),
)]
pub async fn patch(
    repository: web::Data<EnvironmentRepository>,
    id: web::Path<uuid::Uuid>,
    payload: web::Json<EnvironmentUpdatePayload>,
) -> Result<HttpResponse, Error> {
    let environment = repository
        .update(id.into_inner(), payload.into_inner())
        .await;
    if environment.is_err() {
        let error_message = environment.unwrap_err().to_string();
        match error_message.as_str() {
            "No changes to update" => return Err(actix_web::error::ErrorBadRequest(error_message)),
            "Environment not found" => {
                return Err(actix_web::error::ErrorNotFound(error_message));
            }
            "Project Id, name combination already exists" => {
                return Err(actix_web::error::ErrorConflict(error_message));
            }
            _ => return Err(actix_web::error::ErrorInternalServerError(error_message)),
        }
    }
    Ok(HttpResponse::Ok().json(EnvironmentResponse::from(environment.unwrap())))
}

#[utoipa::path(
    delete,
    path = "/environments/{id}",
    tag = "Environments",
    responses(
        (status = 204, description = "Environment deleted", body = ()),
        (status = 404, description = "Environment not found", body = String),
    ),
    params(
        ("id" = String<uuid::Uuid>, Path, description = "Environment ID"),
    )
)]
pub async fn delete(
    repository: web::Data<EnvironmentRepository>,
    id: web::Path<uuid::Uuid>,
) -> Result<HttpResponse, Error> {
    let result = repository.delete(id.into_inner()).await;
    if result.is_err() {
        let error_message = result.unwrap_err().to_string();
        match error_message.as_str() {
            "Environment not found" => {
                return Err(actix_web::error::ErrorNotFound(error_message));
            }
            _ => return Err(actix_web::error::ErrorInternalServerError(error_message)),
        }
    }
    Ok(HttpResponse::NoContent().finish())
}

#[utoipa::path(
    get,
    path = "/environments",
    tag = "Environments",
    responses(
        (status = 200, description = "Environments found", body = Vec<EnvironmentResponse>),
    ),
    params(
        ("project_id" = Option<String>, Query, description = "Filter environments by project ID"),
        ("name" = Option<String>, Query, description = "Filter environments by name"),
        ("description" = Option<String>, Query, description = "Filter environments by description"),
        ("enabled" = Option<bool>, Query, description = "Filter environments by enabled status"),
        ("offset" = Option<u32>, Query, description = "Offset for pagination"),
        ("limit" = Option<u32>, Query, description = "Number of items per page"),
    )
)]
pub async fn list(
    repository: web::Data<EnvironmentRepository>,
    filter: web::Query<EnvironmentFilter>,
    pagination: web::Query<Pagination>,
) -> Result<HttpResponse, Error> {
    let sort = vec![EnvironmentSortOrder::new(
        EnvironmentSortableFields::Id,
        SortOrder::Asc,
    )];
    let environments = repository
        .find(
            filter.into_inner(),
            Some(sort),
            Some(pagination.into_inner()),
        )
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;
    let responses: Vec<EnvironmentResponse> = environments
        .into_iter()
        .map(EnvironmentResponse::from)
        .collect();
    Ok(HttpResponse::Ok().json(responses))
}

pub fn configure_routes(config: &mut actix_web::web::ServiceConfig) {
    config.service(
        web::scope("/environments")
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
