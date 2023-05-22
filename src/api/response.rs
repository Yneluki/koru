#[cfg(feature = "openapi")]
use crate::api::routes::AllGroupsResponse;
#[cfg(feature = "openapi")]
use crate::api::routes::AllUsersResponse;
#[cfg(feature = "openapi")]
use crate::api::routes::ExpensesResponse;
#[cfg(feature = "openapi")]
use crate::api::routes::GroupResponse;
#[cfg(feature = "openapi")]
use crate::api::routes::GroupTokenResponse;
#[cfg(feature = "openapi")]
use crate::api::routes::GroupsResponse;
#[cfg(feature = "openapi")]
use crate::api::routes::SettlementsResponse;
#[cfg(feature = "openapi")]
use crate::domain::usecases::dto::dtos::SettlementDto;
#[cfg(feature = "openapi")]
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(serde::Serialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[cfg_attr(feature = "openapi",  aliases(
    MessageResponse = ResponseMessage<MessageData>,
    ErrorResponse = ResponseMessage<ErrorData>,
    IdResponse = ResponseMessage<IdData>,
    GenerateTokenResponse = ResponseMessage<GroupTokenResponse>,
    GetExpensesResponse = ResponseMessage<ExpensesResponse>,
    GetGroupResponse = ResponseMessage<GroupResponse>,
    GetGroupsResponse = ResponseMessage<GroupsResponse>,
    GetAllGroupsResponse = ResponseMessage<AllGroupsResponse>,
    GetAllUsersResponse = ResponseMessage<AllUsersResponse>,
    GetSettlementsResponse = ResponseMessage<SettlementsResponse>,
    SettlementResponse = ResponseMessage<SettlementDto>,
))]
pub struct ResponseMessage<T> {
    pub success: bool,
    pub data: T,
}

#[derive(serde::Serialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct ErrorData {
    pub error: String,
}

#[derive(serde::Serialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct MessageData {
    pub message: String,
}

#[derive(serde::Serialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct IdData {
    pub id: Uuid,
}

pub fn error(message: &str) -> ResponseMessage<ErrorData> {
    ResponseMessage {
        success: false,
        data: ErrorData {
            error: String::from(message),
        },
    }
}

pub fn ok<T>(data: T) -> ResponseMessage<T> {
    ResponseMessage {
        success: true,
        data,
    }
}

pub fn ok_message(message: &str) -> ResponseMessage<MessageData> {
    ResponseMessage {
        success: true,
        data: MessageData {
            message: String::from(message),
        },
    }
}

pub fn ok_id(id: Uuid) -> ResponseMessage<IdData> {
    ResponseMessage {
        success: true,
        data: IdData { id },
    }
}
