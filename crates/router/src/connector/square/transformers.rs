use error_stack::report;
use serde::{Deserialize, Serialize};
use storage_models::enums::Currency;
use uuid::Uuid;

use crate::{
    consts,
    core::errors,
    types::{self, api, storage::enums},
};

#[derive(Default, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct AmountMoney {
    pub amount: i64,
    pub currency: Currency,
}

#[derive(Debug, Deserialize)]
pub struct PaymentMethodOptions {
    #[serde(rename = "3d_required")]
    pub three_ds: bool,
}

#[derive(Default, Debug, Serialize)]
pub struct SquarePaymentsRequest {
    pub source_id: String,
    pub idempotency_key: String,
    pub amount_money: AmountMoney,
    pub autocomplete: bool,
    pub verification_token: Option<String>,
}

#[derive(Default, Debug, Deserialize)]
pub struct SquarePaymentsResponse {
    pub payment: Option<Payment>,
    pub errors: Option<Vec<SquareErrorResponse>>,
}

#[derive(Default, Debug, Deserialize)]
pub struct ProcessingFee {
    pub effective_at: String,
    pub amount_money: AmountMoney,
}

#[derive(Debug, Deserialize)]
pub struct Payment {
    pub id: String,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub amount_money: AmountMoney,
    pub app_fee_money: Option<AmountMoney>,
    pub delay_duration: Option<String>,
    pub source_type: Option<String>,
    pub card_details: CardDetails,
    pub location_id: Option<String>,
    pub order_id: Option<String>,
    pub reference_id: Option<String>,
    pub risk_evaluation: Option<RiskEvaluation>,
    pub note: Option<String>,
    pub customer_id: Option<String>,
    pub total_money: Option<AmountMoney>,
    pub approved_money: Option<AmountMoney>,
    pub receipt_number: Option<String>,
    pub receipt_url: Option<String>,
    pub delay_action: Option<String>,
    pub delayed_until: Option<String>,
    pub application_details: Option<ApplicationDetails>,
    pub version_token: Option<String>,
    pub processing_key: Option<Vec<ProcessingFee>>,
}

#[derive(Debug, Deserialize)]
pub struct CardDetails {
    pub status: SquarePaymentStatus,
    pub card: Option<Card>,
    pub entry_method: Option<String>,
    pub cvv_status: Option<String>,
    pub avs_status: Option<String>,
    pub statement_description: Option<String>,
    pub auth_result_code: Option<String>,
    pub card_payment_timeline: Option<CardPaymentTimeline>,
}

#[derive(Debug, Deserialize)]
pub struct Card {
    pub card_brand: Option<String>,
    pub last_4: Option<String>,
    pub exp_month: Option<i32>,
    pub exp_year: Option<i32>,
    pub fingerprint: Option<String>,
    pub card_type: Option<String>,
    pub prepaid_type: Option<String>,
    pub bin: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CardPaymentTimeline {
    pub authorized_at: Option<String>,
    pub captured_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RiskEvaluation {
    pub created_at: Option<String>,
    pub risk_level: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApplicationDetails {
    square_product: Option<String>,
    application_id: Option<String>,
}

impl TryFrom<&types::PaymentsAuthorizeRouterData> for SquarePaymentsRequest {
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from(item: &types::PaymentsAuthorizeRouterData) -> Result<Self, Self::Error> {
        let (capture, three_ds_enabled) = match item.payment_method {
            storage_models::enums::PaymentMethodType::Card => {
                let three_ds_enabled = matches!(item.auth_type, enums::AuthenticationType::ThreeDs);
                (
                    matches!(
                        item.request.capture_method,
                        Some(enums::CaptureMethod::Automatic) | None
                    ),
                    three_ds_enabled,
                )
            }
            _ => (false, false),
        };
        let (nonce, verification_token) = match item.request.metadata {
            Some(ref meta) => (meta.clone().session_id, meta.clone().verification_token),
            None => (None, None),
        };
        if (three_ds_enabled && verification_token.is_some()) || !three_ds_enabled {
            Ok(Self {
                source_id: nonce.unwrap_or_default(),
                idempotency_key: Uuid::new_v4().to_string(),
                amount_money: AmountMoney {
                    amount: item.request.amount,
                    currency: item.request.currency,
                },
                autocomplete: capture,
                verification_token,
            })
        }
        else {
            Err(report!(errors::ConnectorError::MissingRequiredField { field_name: "verification_token" }))
        }
    }
}

//TODO: Fill the struct with respective fields
// Auth Struct
pub struct SquareAuthType {
    pub access_token: String,
    pub api_version: String,
}

impl TryFrom<&types::ConnectorAuthType> for SquareAuthType {
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from(auth_type: &types::ConnectorAuthType) -> Result<Self, Self::Error> {
        if let types::ConnectorAuthType::BodyKey { api_key, key1 } = auth_type {
            Ok(Self {
                access_token: api_key.to_string(),
                api_version: key1.to_string(),
            })
        } else {
            Err(errors::ConnectorError::FailedToObtainAuthType)?
        }
    }
}
// PaymentsResponse
//TODO: Append the remaining status flags
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub enum SquarePaymentStatus {
    #[serde(rename = "CAPTURED")]
    Captured,
    #[serde(rename = "VOIDED")]
    Voided,
    #[serde(rename = "FAILED")]
    Failed,
    #[default]
    #[serde(rename = "AUTHORIZED")]
    Authorized,
}

impl From<SquarePaymentStatus> for enums::AttemptStatus {
    fn from(item: SquarePaymentStatus) -> Self {
        match item {
            SquarePaymentStatus::Captured => Self::Charged,
            SquarePaymentStatus::Voided => Self::Voided,
            SquarePaymentStatus::Failed => Self::Failure,
            SquarePaymentStatus::Authorized => Self::Authorized,
        }
    }
}

//TODO: Fill the struct with respective fields

impl<F, T>
    TryFrom<types::ResponseRouterData<F, SquarePaymentsResponse, T, types::PaymentsResponseData>>
    for types::RouterData<F, T, types::PaymentsResponseData>
{
    type Error = error_stack::Report<errors::ParsingError>;
    fn try_from(
        item: types::ResponseRouterData<F, SquarePaymentsResponse, T, types::PaymentsResponseData>,
    ) -> Result<Self, Self::Error> {
        match item.response.payment {
            Some(payment) => Ok(Self {
                status: enums::AttemptStatus::from(payment.card_details.status),
                response: Ok(types::PaymentsResponseData::TransactionResponse {
                    resource_id: types::ResponseId::ConnectorTransactionId(payment.id),
                    redirection_data: None,
                    redirect: false,
                    mandate_reference: None,
                    connector_metadata: None,
                }),
                ..item.data
            }),
            _ => Ok(Self {
                status: enums::AttemptStatus::AuthenticationFailed,
                response: Err(types::ErrorResponse {
                    code: consts::NO_ERROR_CODE.to_string(),
                    message: String::from(""),
                    reason: None,
                    status_code: item.http_code,
                }),
                ..item.data
            }),
        }
    }
}

//TODO: Fill the struct with respective fields
// REFUND :
// Type definition for RefundRequest
#[derive(Default, Debug, Serialize)]
pub struct SquareRefundRequest {
    pub amount_money: AmountMoney,
    pub idempotency_key: String,
    pub payment_id: String,
}
#[derive(Debug, Deserialize)]
pub struct SquareRefundResponse {
    pub errors: Option<Vec<SquareErrorResponse>>,
    pub refund: Refund,
}
#[derive(Debug, Deserialize)]

pub struct Refund {
    pub id: String,
    pub status: RefundStatus,
    pub amount_money: AmountMoney,
    pub payment_id: String,
    pub order_id: String,
    pub created_at: String,
    pub updated_at: String,
    pub processing_fee: Option<Vec<ProcessingFee>>,
    pub location_id: String,
    pub destination_type: String,
}

impl<F> TryFrom<&types::RefundsRouterData<F>> for SquareRefundRequest {
    type Error = error_stack::Report<errors::ParsingError>;
    fn try_from(item: &types::RefundsRouterData<F>) -> Result<Self, Self::Error> {
        let amount_money = AmountMoney {
            amount: item.request.amount,
            currency: item.request.currency,
        };

        let idempotency_key: String = Uuid::new_v4().to_string();

        let payment_id = item.request.connector_transaction_id.to_string();

        Ok(Self {
            amount_money,
            idempotency_key,
            payment_id,
        })
    }
}

#[derive(Debug, Serialize, Default, Deserialize, Clone)]
pub enum RefundStatus {
    COMPLETED,
    FAILED,
    REJECTED,
    #[default]
    PENDING,
}

impl From<RefundStatus> for enums::RefundStatus {
    fn from(item: RefundStatus) -> Self {
        match item {
            RefundStatus::COMPLETED => Self::Success,
            RefundStatus::FAILED => Self::Failure,
            RefundStatus::PENDING => Self::Pending,
            RefundStatus::REJECTED => Self::TransactionFailure,
        }
    }
}

impl TryFrom<types::RefundsResponseRouterData<api::Execute, SquareRefundResponse>>
    for types::RefundsRouterData<api::Execute>
{
    type Error = error_stack::Report<errors::ParsingError>;
    fn try_from(
        item: types::RefundsResponseRouterData<api::Execute, SquareRefundResponse>,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            response: Ok(types::RefundsResponseData {
                connector_refund_id: item.response.refund.id,
                refund_status: enums::RefundStatus::from(item.response.refund.status),
            }),
            ..item.data
        })
    }
}

impl TryFrom<types::RefundsResponseRouterData<api::RSync, SquareRefundResponse>>
    for types::RefundsRouterData<api::RSync>
{
    type Error = error_stack::Report<errors::ParsingError>;
    fn try_from(
        item: types::RefundsResponseRouterData<api::RSync, SquareRefundResponse>,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            response: Ok(types::RefundsResponseData {
                connector_refund_id: item.response.refund.id,
                refund_status: enums::RefundStatus::from(item.response.refund.status),
            }),
            ..item.data
        })
    }
}

#[derive(Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct SquareErrorResponse {
    pub category: String,
    pub code: String,
    pub detail: Option<String>,
    pub field: Option<String>,
}
