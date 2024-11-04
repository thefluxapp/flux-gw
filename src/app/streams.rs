use axum::{extract::State, routing::get, Json, Router};
use get_streams::Response;

use super::{error::AppError, state::AppState};

pub fn router() -> Router<AppState> {
    Router::new().route("/", get(get_streams))
}

async fn get_streams(
    State(AppState {
        streams_service_client,
        users_service_client,
        ..
    }): State<AppState>,
) -> Result<Json<Response>, AppError> {
    let get_last_streams_response = streams_service_client
        .clone()
        .get_last_streams(flux_core_api::GetLastStreamsRequest::default())
        .await?
        .into_inner();

    let get_streams_response = streams_service_client
        .clone()
        .get_streams(flux_core_api::GetStreamsRequest {
            stream_ids: get_last_streams_response.stream_ids,
        })
        .await?
        .into_inner();

    let get_users_response = users_service_client
        .clone()
        .get_users(flux_auth_api::GetUsersRequest {
            user_ids: get_streams_response
                .streams
                .iter()
                .map(|m| m.user_ids.clone())
                .flatten()
                .collect(),
        })
        .await?
        .into_inner();

    Ok(Json((get_streams_response, get_users_response).try_into()?))
}

mod get_streams {
    use std::collections::HashMap;

    use anyhow::{anyhow, Error};
    use flux_auth_api::get_users_response;
    use serde::Serialize;

    #[derive(Serialize)]
    pub struct Response {
        pub streams: Vec<Stream>,
    }

    #[derive(Serialize)]
    pub struct Stream {
        pub stream_id: String,
        pub message_id: String,
        pub text: Option<String>,
        pub users: Vec<User>,
    }

    #[derive(Serialize)]
    pub struct User {
        user_id: String,
        name: String,
    }

    impl
        TryFrom<(
            flux_core_api::GetStreamsResponse,
            flux_auth_api::GetUsersResponse,
        )> for Response
    {
        type Error = Error;

        fn try_from(
            (get_streams_response, get_users_response): (
                flux_core_api::GetStreamsResponse,
                flux_auth_api::GetUsersResponse,
            ),
        ) -> Result<Self, Self::Error> {
            let users: HashMap<String, get_users_response::User> = get_users_response
                .users
                .into_iter()
                .map(|v| (v.user_id().into(), v))
                .collect();

            Ok(Self {
                streams: get_streams_response
                    .streams
                    .iter()
                    .map(|m| -> Result<Stream, Self::Error> {
                        Ok(Stream {
                            stream_id: m.stream_id().into(),
                            message_id: m.message_id().into(),
                            text: m.text.clone(),
                            users: m
                                .user_ids
                                .iter()
                                .map(|user_id| -> Result<User, Self::Error> {
                                    users.get(user_id).try_into()
                                })
                                .collect::<Result<Vec<User>, Self::Error>>()?,
                        })
                    })
                    .collect::<Result<Vec<Stream>, Self::Error>>()?,
            })
        }
    }

    impl TryFrom<Option<&get_users_response::User>> for User {
        type Error = Error;

        fn try_from(user: Option<&get_users_response::User>) -> Result<Self, Self::Error> {
            let user = user.ok_or(anyhow!("user not found"))?.to_owned();

            Ok(Self {
                user_id: user.user_id().into(),
                name: user.name().into(),
            })
        }
    }
}
