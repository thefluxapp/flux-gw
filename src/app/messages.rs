use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use create_message::Request;
use flux_auth_api::GetUsersRequest;
use flux_core_api::{CreateMessageRequest, GetMessageRequest, GetStreamsRequest};
use uuid::Uuid;

use super::{error::AppError, state::AppState, user::AppUser};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/:message_id", get(get_message))
        .route("/", post(create_message))
}

async fn get_message(
    Path(message_id): Path<Uuid>,
    State(AppState {
        messages_service_client,
        users_service_client,
        streams_service_client,
        ..
    }): State<AppState>,
) -> Result<Json<get_message::Res>, AppError> {
    // TODO: make requests not seq

    let get_message_response = messages_service_client
        .clone()
        .get_message(GetMessageRequest {
            message_id: Some(message_id.into()),
        })
        .await?
        .into_inner();

    let mut get_streams_request = GetStreamsRequest::default();

    for message in &get_message_response.messages {
        if let Some(stream_id) = message.stream_id.clone() {
            get_streams_request.stream_ids.push(stream_id);
        }
    }

    if let Some(message) = get_message_response.message.clone() {
        if let Some(stream_id) = message.stream_id {
            get_streams_request.stream_ids.push(stream_id);
        }
    }

    let get_streams_response = streams_service_client
        .clone()
        .get_streams(get_streams_request)
        .await?
        .into_inner();

    let mut get_users_request = GetUsersRequest::default();

    for message in &get_message_response.messages {
        get_users_request.user_ids.push(message.user_id().into());
    }

    for stream in &get_streams_response.streams {
        get_users_request.user_ids.extend(stream.user_ids.clone());
    }

    let get_users_response = users_service_client
        .clone()
        .get_users(get_users_request)
        .await?
        .into_inner();

    Ok(Json(
        (
            get_message_response,
            get_users_response,
            get_streams_response,
        )
            .try_into()?,
    ))
}

mod get_message {
    use std::collections::HashMap;

    use anyhow::{anyhow, Error};
    use flux_auth_api::{get_users_response, GetUsersResponse};
    use flux_core_api::{
        get_message_response, get_streams_response, GetMessageResponse, GetStreamsResponse,
    };
    use serde::Serialize;

    #[derive(Serialize)]
    pub struct Res {
        pub message: Message,
        pub messages: Vec<Message>,
    }

    #[derive(Serialize)]
    pub struct Stream {
        pub stream_id: String,
        pub text: Option<String>,
        pub users: Vec<User>,
    }

    #[derive(Serialize)]
    pub struct Message {
        pub message_id: String,
        pub stream: Option<Stream>,
        pub text: String,
        pub user: User,
    }

    #[derive(Serialize)]
    pub struct User {
        pub user_id: String,
        pub name: String,
        pub first_name: String,
        pub last_name: String,
    }

    type Users = HashMap<String, get_users_response::User>;
    type Streams = HashMap<String, get_streams_response::Stream>;

    impl TryFrom<(GetMessageResponse, GetUsersResponse, GetStreamsResponse)> for Res {
        type Error = Error;

        fn try_from(
            (get_message_response, get_users_response, get_streams_response): (
                GetMessageResponse,
                GetUsersResponse,
                GetStreamsResponse,
            ),
        ) -> Result<Self, Self::Error> {
            let users: Users = get_users_response
                .users
                .into_iter()
                .map(|v| (v.user_id().into(), v))
                .collect();

            let streams: Streams = get_streams_response
                .streams
                .into_iter()
                .map(|v| (v.stream_id().into(), v))
                .collect();

            let message = get_message_response
                .message
                .ok_or(anyhow!("message not found"))?;

            Ok(Self {
                message: (message.clone(), &users, streams.get(message.stream_id())).try_into()?,

                messages: get_message_response
                    .messages
                    .into_iter()
                    .map(|m| -> Result<Message, Self::Error> {
                        (m.clone(), &users, streams.get(m.stream_id())).try_into()
                    })
                    .collect::<Result<Vec<Message>, Self::Error>>()?,
            })
        }
    }

    impl
        TryFrom<(
            get_message_response::Message,
            &Users,
            Option<&get_streams_response::Stream>,
        )> for Message
    {
        type Error = Error;

        fn try_from(
            (message, users, stream): (
                get_message_response::Message,
                &Users,
                Option<&get_streams_response::Stream>,
            ),
        ) -> Result<Self, Self::Error> {
            let user = users
                .get(&message.user_id().to_string())
                .ok_or(anyhow!("user not found"))?
                .to_owned();

            Ok(Self {
                message_id: message.message_id().into(),
                text: message.text().into(),
                user: user.into(),
                stream: match stream {
                    Some(stream) => Some((stream.to_owned(), users).try_into()?),
                    None => None,
                },
            })
        }
    }

    impl TryFrom<(get_streams_response::Stream, &Users)> for Stream {
        type Error = Error;

        fn try_from(
            (stream, users): (get_streams_response::Stream, &Users),
        ) -> Result<Self, Self::Error> {
            Ok(Self {
                stream_id: stream.stream_id().into(),
                text: stream.text,
                users: stream
                    .user_ids
                    .iter()
                    .map(|user_id| -> Result<User, Error> {
                        let user = users
                            .get(&user_id.to_string())
                            .ok_or(anyhow!("user not found"))?
                            .to_owned();

                        Ok(User {
                            user_id: user.user_id().into(),
                            name: user.name().into(),
                            first_name: user.first_name().into(),
                            last_name: user.last_name().into(),
                        })
                    })
                    .collect::<Result<Vec<User>, Self::Error>>()?,
            })
        }
    }

    impl From<get_users_response::User> for User {
        fn from(user: get_users_response::User) -> Self {
            Self {
                user_id: user.user_id().into(),
                name: user.name().into(),
                first_name: user.first_name().into(),
                last_name: user.last_name().into(),
            }
        }
    }
}

async fn create_message(
    State(AppState {
        messages_service_client,
        ..
    }): State<AppState>,
    user: AppUser,
    Json(data): Json<Request>,
) -> Result<Json<create_message::Response>, AppError> {
    let res = messages_service_client
        .clone()
        .create_message(CreateMessageRequest {
            text: Some(data.text),
            message_id: match data.message_id {
                Some(message_id) => Some(message_id.into()),
                None => None,
            },
            user_id: Some(user.id.into()),
        })
        .await?
        .into_inner();

    Ok(Json(res.into()))
}

mod create_message {
    use flux_core_api::CreateMessageResponse;
    use serde::{Deserialize, Serialize};
    use uuid::Uuid;

    #[derive(Deserialize, Debug)]
    pub struct Request {
        pub text: String,
        pub message_id: Option<Uuid>,
    }

    #[derive(Serialize)]
    pub struct Response {
        pub message: Message,
    }

    #[derive(Serialize)]
    pub struct Message {
        pub message_id: String,
    }

    impl From<CreateMessageResponse> for Response {
        fn from(res: CreateMessageResponse) -> Self {
            Self {
                message: Message {
                    message_id: res.message_id().into(),
                },
            }
        }
    }
}
