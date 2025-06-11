use axum::{extract::State, routing::get, Json, Router};
use flux_messages_api::GetUserStreamsRequest;
use get_last_streams::Response;

use crate::app::locale::AppLocale;

use super::{error::AppError, state::AppState, user::AppUser};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(get_last_streams))
        .route("/my", get(get_user_streams))
}

// TODO: make requests async

async fn get_last_streams(
    State(AppState {
        streams_service_client,
        users_service_client,
        ..
    }): State<AppState>,
    locale: AppLocale,
) -> Result<Json<Response>, AppError> {
    let get_last_streams_response = streams_service_client
        .clone()
        .get_last_streams(flux_messages_api::GetLastStreamsRequest {
            locale: Some(locale.to_string()),
        })
        .await?
        .into_inner();

    let get_streams_response = streams_service_client
        .clone()
        .get_streams(flux_messages_api::GetStreamsRequest {
            stream_ids: get_last_streams_response.stream_ids,
        })
        .await?
        .into_inner();

    let get_users_response = users_service_client
        .clone()
        .get_users(flux_users_api::GetUsersRequest {
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

mod get_last_streams {
    use std::collections::HashMap;

    use flux_users_api::get_users_response;
    use serde::Serialize;

    use crate::app::error::AppError;

    #[derive(Serialize)]
    pub struct Response {
        streams: Vec<Stream>,
    }

    #[derive(Serialize)]
    struct Stream {
        stream_id: String,
        message_id: String,
        text: Option<String>,
        users: Vec<User>,
    }

    #[derive(Serialize)]
    struct User {
        user_id: String,
        name: String,
        first_name: String,
        last_name: String,
        abbr: String,
        color: String,
    }

    impl
        TryFrom<(
            flux_messages_api::GetStreamsResponse,
            flux_users_api::GetUsersResponse,
        )> for Response
    {
        type Error = AppError;

        fn try_from(
            (get_streams_response, get_users_response): (
                flux_messages_api::GetStreamsResponse,
                flux_users_api::GetUsersResponse,
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
        type Error = AppError;

        fn try_from(user: Option<&get_users_response::User>) -> Result<Self, Self::Error> {
            let user = user.ok_or(AppError::NoEntity)?.to_owned();

            Ok(Self {
                user_id: user.user_id().into(),
                name: user.name().into(),
                first_name: user.first_name().into(),
                last_name: user.last_name().into(),
                abbr: user.abbr().into(),
                color: user.color().into(),
            })
        }
    }
}

async fn get_user_streams(
    State(AppState {
        streams_service_client,
        users_service_client,
        ..
    }): State<AppState>,
    user: AppUser,
) -> Result<Json<get_user_streams::Res>, AppError> {
    let get_user_streams_response = streams_service_client
        .clone()
        .get_user_streams(GetUserStreamsRequest {
            user_id: Some(user.id.into()),
        })
        .await?
        .into_inner();

    let get_streams_response = streams_service_client
        .clone()
        .get_streams(flux_messages_api::GetStreamsRequest {
            stream_ids: get_user_streams_response.stream_ids,
        })
        .await?
        .into_inner();

    let get_users_response = users_service_client
        .clone()
        .get_users(flux_users_api::GetUsersRequest {
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

mod get_user_streams {
    use std::collections::HashMap;

    use flux_users_api::get_users_response;
    use serde::Serialize;

    use crate::app::error::AppError;

    #[derive(Serialize)]
    pub struct Res {
        streams: Vec<Stream>,
    }

    #[derive(Serialize)]
    pub struct Stream {
        stream_id: String,
        message_id: String,
        text: Option<String>,
        users: Vec<User>,
    }

    #[derive(Serialize)]
    pub struct User {
        user_id: String,
        name: String,
        first_name: String,
        last_name: String,
        abbr: String,
        color: String,
    }

    impl
        TryFrom<(
            flux_messages_api::GetStreamsResponse,
            flux_users_api::GetUsersResponse,
        )> for Res
    {
        type Error = AppError;

        fn try_from(
            (get_streams_response, get_users_response): (
                flux_messages_api::GetStreamsResponse,
                flux_users_api::GetUsersResponse,
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
        type Error = AppError;

        fn try_from(user: Option<&get_users_response::User>) -> Result<Self, Self::Error> {
            let user = user.ok_or(AppError::NoEntity)?.to_owned();

            Ok(Self {
                user_id: user.user_id().into(),
                name: user.name().into(),
                first_name: user.first_name().into(),
                last_name: user.last_name().into(),
                abbr: user.abbr().into(),
                color: user.color().into(),
            })
        }
    }
}
