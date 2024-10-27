use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};
use flux_auth_api::GetUsersRequest;
use flux_core_api::GetMessageRequest;
use get_messages::Response;
use uuid::Uuid;

use super::{error::AppError, state::AppState};

pub fn router() -> Router<AppState> {
    Router::new().route("/:message_id", get(get_messages))
}

async fn get_messages(
    Path(message_id): Path<Uuid>,
    State(AppState {
        messages_service_client,
        users_service_client,
        ..
    }): State<AppState>,
) -> Result<Json<Response>, AppError> {
    // TODO: make requests not seq

    let get_message_response = messages_service_client
        .clone()
        .get_message(GetMessageRequest {
            message_id: Some(message_id.into()),
        })
        .await?
        .into_inner();

    // let get_messages_response = messages_service_client
    //     .clone()
    //     .get_messages(GetMessagesRequest {
    //         message_ids: get_message_response.message_ids.clone(),
    //     })
    //     .await?
    //     .into_inner();

    let get_users_response = users_service_client
        .clone()
        .get_users(GetUsersRequest {
            user_ids: get_message_response
                .messages
                .iter()
                .map(|m| m.user_id().into())
                .collect(),
        })
        .await?
        .into_inner();

    Ok(Json((get_message_response, get_users_response).try_into()?))
}

mod get_messages {
    use std::collections::HashMap;

    use anyhow::{anyhow, Error};
    use flux_auth_api::{get_users_response, GetUsersResponse};
    use flux_core_api::{get_message_response, GetMessageResponse};
    use serde::Serialize;

    #[derive(Serialize)]
    pub struct Response {
        pub message: Message,
        pub messages: Vec<Message>,
    }

    #[derive(Serialize)]
    pub struct Stream {
        pub stream_id: String,
        pub text: Option<String>,
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
    }

    type Users = HashMap<String, get_users_response::User>;

    impl TryFrom<(GetMessageResponse, GetUsersResponse)> for Response {
        type Error = Error;

        fn try_from(
            (get_message_response, get_users_response): (GetMessageResponse, GetUsersResponse),
        ) -> Result<Self, Self::Error> {
            let users: Users = get_users_response
                .users
                .into_iter()
                .map(|v| (v.user_id().into(), v))
                .collect();

            let message = get_message_response
                .message
                .ok_or(anyhow!("message not found"))?;

            Ok(Self {
                message: (message.clone(), users.get(message.user_id())).try_into()?,

                messages: get_message_response
                    .messages
                    .into_iter()
                    .map(|m| -> Result<Message, Self::Error> {
                        (m.clone(), users.get(m.user_id())).try_into()
                    })
                    .collect::<Result<Vec<Message>, Self::Error>>()?,
            })
        }
    }

    impl
        TryFrom<(
            get_message_response::Message,
            Option<&get_users_response::User>,
        )> for Message
    {
        type Error = Error;

        fn try_from(
            (message, user): (
                get_message_response::Message,
                Option<&get_users_response::User>,
            ),
        ) -> Result<Self, Self::Error> {
            let user = user.ok_or(anyhow!("user not found"))?.to_owned();
            Ok(Self {
                message_id: message.message_id().into(),
                text: message.text().into(),
                user: user.into(),
                stream: match message.stream {
                    Some(stream) => Some(stream.into()),
                    None => None,
                },
            })
        }
    }

    impl From<get_message_response::Stream> for Stream {
        fn from(stream: get_message_response::Stream) -> Self {
            Self {
                stream_id: stream.stream_id().into(),
                text: stream.text,
            }
        }
    }

    impl From<get_users_response::User> for User {
        fn from(user: get_users_response::User) -> Self {
            Self {
                user_id: user.user_id().into(),
                name: user.name().into(),
            }
        }
    }
}