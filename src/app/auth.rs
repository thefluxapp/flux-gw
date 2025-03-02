use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{error::AppError, state::AppState, user::AppUser};

pub mod settings;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/login", post(login))
        .route("/join", post(join))
        .route("/complete", post(complete))
        .route("/me", get(me))
}

async fn login(
    State(AppState {
        auth_service_client,
        ..
    }): State<AppState>,
    Json(data): Json<Value>,
) -> Result<Json<login::Response>, AppError> {
    let request = flux_users_api::LoginRequest {
        request: Some(data.to_string()),
    };
    let response = auth_service_client
        .clone()
        .login(request)
        .await?
        .into_inner();

    Ok(Json(response.into()))
}

mod login {
    use flux_users_api::LoginResponse;
    use serde::Serialize;

    #[derive(Serialize)]
    pub struct Response {
        jwt: String,
    }

    impl From<LoginResponse> for Response {
        fn from(res: LoginResponse) -> Self {
            Self {
                jwt: res.jwt().into(),
            }
        }
    }
}

async fn join(
    State(AppState {
        auth_service_client,
        ..
    }): State<AppState>,
    Json(data): Json<JoinRequest>,
) -> Result<Json<Value>, AppError> {
    let request: flux_users_api::JoinRequest = data.into();
    let response = auth_service_client
        .clone()
        .join(request)
        .await?
        .into_inner();

    Ok(Json(serde_json::from_str(response.response())?))
}

#[derive(Deserialize)]
struct JoinRequest {
    pub email: Option<String>,
}

impl From<JoinRequest> for flux_users_api::JoinRequest {
    fn from(request: JoinRequest) -> Self {
        Self {
            email: request.email,
        }
    }
}

async fn complete(
    State(AppState {
        auth_service_client,
        ..
    }): State<AppState>,
    Json(data): Json<Value>,
) -> Result<Json<CompleteResponse>, AppError> {
    let request = flux_users_api::CompleteRequest {
        request: Some(data.to_string()),
    };
    let response = auth_service_client
        .clone()
        .complete(request)
        .await?
        .into_inner();

    Ok(Json(response.into()))
}

impl Into<CompleteResponse> for flux_users_api::CompleteResponse {
    fn into(self) -> CompleteResponse {
        CompleteResponse {
            jwt: self.jwt().into(),
        }
    }
}

#[derive(Serialize)]
struct CompleteResponse {
    pub jwt: String,
}

async fn me(
    State(AppState {
        auth_service_client,
        ..
    }): State<AppState>,
    user: Option<AppUser>,
) -> Result<Json<me::Response>, AppError> {
    let response = match user {
        Some(user) => {
            let request = flux_users_api::MeRequest {
                user_id: Some(user.id.into()),
            };

            auth_service_client
                .clone()
                .me(request)
                .await?
                .into_inner()
                .try_into()?
        }
        None => me::Response { user: None },
    };

    Ok(Json(response))
}

mod me {
    use serde::Serialize;

    use crate::app::error::AppError;

    #[derive(Serialize)]
    pub struct Response {
        pub user: Option<User>,
    }

    #[derive(Serialize)]
    pub struct User {
        pub user_id: String,
        pub name: String,
        pub first_name: String,
        pub last_name: String,
        pub abbr: String,
        pub color: String,
    }

    impl TryFrom<flux_users_api::MeResponse> for Response {
        type Error = AppError;

        fn try_from(res: flux_users_api::MeResponse) -> Result<Self, Self::Error> {
            let user = res.user.ok_or(AppError::NoEntity)?;

            Ok(Response {
                user: Some(User {
                    user_id: user.user_id().into(),
                    name: user.name().into(),
                    first_name: user.first_name().into(),
                    last_name: user.last_name().into(),
                    abbr: user.abbr().into(),
                    color: user.color().into(),
                }),
            })
        }
    }
}
