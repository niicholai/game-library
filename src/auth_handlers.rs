use axum::{
    extract::{Extension, State, Path},
    http::StatusCode,
    response::Json,
};
use axum_macros::debug_handler;
use crate::{
    auth::{LoginRequest, CreateUserRequest, LoginResponse, UserResponse, User},
    handlers::{AppState, ApiResponse},
};

#[debug_handler]
#[allow(unused_variables)]
pub async fn login(
    State(state): State<AppState>,
    Json(request): Json<LoginRequest>,
) -> Result<Json<ApiResponse<LoginResponse>>, StatusCode> {
    match state.auth_service.login(request).await {
        Ok((user, session)) => {
            let response = LoginResponse {
                user: user.into(),
                token: session.token,
            };
            Ok(Json(ApiResponse::success(response)))
        }
        Err(_) => Err(StatusCode::UNAUTHORIZED),
    }
}

#[debug_handler]
#[allow(unused_variables)]
pub async fn logout(
    State(_state): State<AppState>,
    Extension(_user): Extension<User>,
) -> Result<StatusCode, StatusCode> {
    Ok(StatusCode::OK)
}

#[debug_handler]
pub async fn me(
    Extension(user): Extension<User>,
) -> Json<ApiResponse<UserResponse>> {
    Json(ApiResponse::success(user.into()))
}

#[debug_handler]
#[allow(unused_variables)]
pub async fn create_user(
    State(state): State<AppState>,
    Extension(_admin): Extension<User>,
    Json(request): Json<CreateUserRequest>,
) -> Result<Json<ApiResponse<UserResponse>>, StatusCode> {
    match state.auth_service.create_user(request).await {
        Ok(user) => Ok(Json(ApiResponse::success(user.into()))),
        Err(_) => Err(StatusCode::BAD_REQUEST),
    }
}

#[debug_handler]
#[allow(unused_variables)]
pub async fn list_users(
    State(state): State<AppState>,
    Extension(_admin): Extension<User>,
) -> Result<Json<ApiResponse<Vec<UserResponse>>>, StatusCode> {
    match state.auth_service.get_all_users().await {
        Ok(users) => {
            let user_responses: Vec<UserResponse> = users.into_iter().map(|u| u.into()).collect();
            Ok(Json(ApiResponse::success(user_responses)))
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[debug_handler]
#[allow(unused_variables)]
pub async fn delete_user(
    State(state): State<AppState>,
    Extension(_admin): Extension<User>,
    Path(user_id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    match state.auth_service.delete_user(&user_id).await {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}
