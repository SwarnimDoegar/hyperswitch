use serde::{Deserialize, Serialize};
use crate::{core::errors,types::{self,api, storage::enums}};

//TODO: Fill the struct with respective fields
#[derive(Default, Debug, Serialize)]
pub struct SquarePaymentsRequest {
    pub source_id: String,
    pub idempotency_key: String,
    pub amount_money: Amount
}

#[derive(Default, Debug, Serialize)]
pub struct Amount {
    amount: i64,
    currency: String
}

impl TryFrom<&types::PaymentsAuthorizeRouterData> for SquarePaymentsRequest  {
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from(_item: &types::PaymentsAuthorizeRouterData) -> Result<Self,Self::Error> {
        let 
        Ok(Self {
            source_id: item.request.source_id,
            idempotency_key: item.request.idempotency_key,

        })
    }
}

//TODO: Fill the struct with respective fields
// Auth Struct
pub struct SquareAuthType {
    pub access_token: String,
    pub api_version: String,
}

impl TryFrom<&types::ConnectorAuthType> for SquareAuthType  {
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
#[serde(rename_all = "lowercase")]
pub enum SquarePaymentStatus {
    Succeeded,
    Failed,
    #[default]
    Processing,
}

impl From<SquarePaymentStatus> for enums::AttemptStatus {
    fn from(item: SquarePaymentStatus) -> Self {
        match item {
            SquarePaymentStatus::Succeeded => Self::Charged,
            SquarePaymentStatus::Failed => Self::Failure,
            SquarePaymentStatus::Processing => Self::Authorizing,
        }
    }
}

//TODO: Fill the struct with respective fields
#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SquarePaymentsResponse {
    pub payment: Option<Payment>,
    pub errors : Option<SquareErrorResponse>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Payment{
    pub id: String,
    pub created_at: Option<i64>,
    pub updated_at: Option<i64>,
    pub amount_money: Amount,
    pub app_fee_money: Option<Amount>,
    pub delay_duration: Option<String>,
    pub source_type: Option<String>,
    pub card_details: Option<CardDetails>,
    pub location_id: Option<String>,
    pub order_id: Option<String>,
    pub reference_id: Option<String>,
    pub risk_evaluation: Option<RiskEvaluation>,
    pub note: Option<String>,
    pub customer_id: Option<String>,
    pub total_money: Option<Amount>,
    pub approved_money: Option<Amount>,
    pub receipt_number: Option<String>,
    pub receipt_url: Option<String>,
    pub delay_action: Option<String>,
    pub delayed_until: Option<i64>,
    pub application_details: Option<ApplicationDetails>,
    pub version_token: Option<String>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CardDetails{
    pub status: Option<SquareStatus>,
    pub card: Option<Card>,
    pub entry_method: Option<String>,
    pub cvv_status: Option<String>,
    pub avs_status: Option<String>,
    pub statement_description: Option<String>,
    pub auth_result_code: Option<String>,
    pub statement_description: Option<String>,
    pub card_payment_timeline: Option<CardPaymentTimeline>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Card{
    pub card_brand: Option<String>,
    pub last_4: Option<String>,
    pub exp_month: Option<i32>,
    pub exp_year: Option<i32>,
    pub fingerprint: Option<String>,
    pub card_type: Option<String>,
    pub prepaid_type: Option<String>,
    pub bin: Option<String>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CardPaymentTimeline{
    authorized_at: Option<i64>,
    captured_at: Option<i64>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RiskEvaluation{
    created_at: Option<i64>,
    risk_level: Option<String>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApplicationDetails{
    square_product: Option<String>,
    application_id: Option<String>
}

impl<F,T> TryFrom<types::ResponseRouterData<F, SquarePaymentsResponse, T, types::PaymentsResponseData>> for types::RouterData<F, T, types::PaymentsResponseData> {
    type Error = error_stack::Report<errors::ParsingError>;
    fn try_from(item: types::ResponseRouterData<F, SquarePaymentsResponse, T, types::PaymentsResponseData>) -> Result<Self,Self::Error> {
        Ok(Self {
            status: enums::AttemptStatus::from(item.response.status),
            response: Ok(types::PaymentsResponseData::TransactionResponse {
                resource_id: types::ResponseId::ConnectorTransactionId(item.response.id),
                redirection_data: None,
                redirect: false,
                mandate_reference: None,
                connector_metadata: None,
            }),
            ..item.data
        })
    }
}

//TODO: Fill the struct with respective fields
// REFUND :
// Type definition for RefundRequest
#[derive(Default, Debug, Serialize)]
pub struct SquareRefundRequest {}

impl<F> TryFrom<&types::RefundsRouterData<F>> for SquareRefundRequest {
    type Error = error_stack::Report<errors::ParsingError>;
    fn try_from(_item: &types::RefundsRouterData<F>) -> Result<Self,Self::Error> {
       todo!()
    }
}

// Type definition for Refund Response

#[allow(dead_code)]
#[derive(Debug, Serialize, Default, Deserialize, Clone)]
pub enum RefundStatus {
    Succeeded,
    Failed,
    #[default]
    Processing,
}

impl From<RefundStatus> for enums::RefundStatus {
    fn from(item: RefundStatus) -> Self {
        match item {
            RefundStatus::Succeeded => Self::Success,
            RefundStatus::Failed => Self::Failure,
            RefundStatus::Processing => Self::Pending,
            //TODO: Review mapping
        }
    }
}

//TODO: Fill the struct with respective fields
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct RefundResponse {
}

impl TryFrom<types::RefundsResponseRouterData<api::Execute, RefundResponse>>
    for types::RefundsRouterData<api::Execute>
{
    type Error = error_stack::Report<errors::ParsingError>;
    fn try_from(
        _item: types::RefundsResponseRouterData<api::Execute, RefundResponse>,
    ) -> Result<Self, Self::Error> {
        todo!()
    }
}

impl TryFrom<types::RefundsResponseRouterData<api::RSync, RefundResponse>> for types::RefundsRouterData<api::RSync>
{
    type Error = error_stack::Report<errors::ParsingError>;
    fn try_from(_item: types::RefundsResponseRouterData<api::RSync, RefundResponse>) -> Result<Self,Self::Error> {
         todo!()
    }
 }

#[derive(Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct SquareErrorResponse {
    category : String,
    code : String,
    detail : Option<String>,
    field : Option<String>,
}
