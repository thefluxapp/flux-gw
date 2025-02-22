use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use flux_notify_api::{CreateWebPushRequest, GetVapidRequest, GetWebPushesRequest};

use super::{error::AppError, state::AppState, user::AppUser};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/vapid", get(get_vapid))
        .route("/", post(create_push))
        .route("/", get(get_pushes))
}

async fn get_vapid(
    State(AppState {
        push_service_client,
        ..
    }): State<AppState>,
) -> Result<Json<get_vapid::Response>, AppError> {
    let res = push_service_client
        .clone()
        .get_vapid(GetVapidRequest {})
        .await?
        .into_inner();

    Ok(Json(res.into()))
}

mod get_vapid {
    use flux_notify_api::GetVapidResponse;
    use serde::Serialize;

    #[derive(Serialize)]
    pub(super) struct Response {
        public_key: String,
    }

    impl From<GetVapidResponse> for Response {
        fn from(res: GetVapidResponse) -> Self {
            Self {
                public_key: res.public_key().into(),
            }
        }
    }
}

async fn create_push(
    State(AppState {
        push_service_client,
        ..
    }): State<AppState>,
    user: AppUser,
    Json(req): Json<create_push::Request>,
) -> Result<Json<create_push::Response>, AppError> {
    let res = push_service_client
        .clone()
        .create_web_push(CreateWebPushRequest {
            endpoint: Some(req.endpoint),
            authentication_secret: Some(req.keys.authentication_secret),
            public_key: Some(req.keys.public_key),
            device_id: Some(req.device_id),
            user_id: Some(user.id.into()),
        })
        .await?
        .into_inner();

    Ok(Json(res.into()))
}

mod create_push {
    use flux_notify_api::CreateWebPushResponse;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize)]
    pub(super) struct Response {}

    #[derive(Deserialize, Debug)]
    pub(super) struct Request {
        pub endpoint: String,
        pub device_id: String,
        pub keys: Keys,
        // pub authentication_secret: String,
        // pub public_key: String,
    }

    #[derive(Deserialize, Debug)]
    pub(super) struct Keys {
        #[serde(rename = "auth")]
        pub authentication_secret: String,
        #[serde(rename = "p256dh")]
        pub public_key: String,
    }

    impl From<CreateWebPushResponse> for Response {
        fn from(_: CreateWebPushResponse) -> Self {
            Self {}
        }
    }
}

async fn get_pushes(
    State(AppState {
        push_service_client,
        ..
    }): State<AppState>,
    user: AppUser,
) -> Result<Json<get_pushes::Response>, AppError> {
    let res = push_service_client
        .clone()
        .get_web_pushes(GetWebPushesRequest {
            user_id: Some(user.id.into()),
        })
        .await?
        .into_inner();

    Ok(Json(res.into()))
}

mod get_pushes {
    use flux_notify_api::GetWebPushesResponse;
    use serde::Serialize;

    #[derive(Serialize)]
    pub struct Response {
        pub device_ids: Vec<String>,
    }

    impl From<GetWebPushesResponse> for Response {
        fn from(res: GetWebPushesResponse) -> Self {
            Self {
                device_ids: res.device_ids,
            }
        }
    }
}
