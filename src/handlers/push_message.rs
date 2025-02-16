use {
    crate::{
        blob::ENCRYPTED_FLAG,
        error::{
            Error::{ClientNotFound, Store},
            Result,
        },
        handlers::{Response, DECENTRALIZED_IDENTIFIER_PREFIX},
        increment_counter,
        log::prelude::*,
        middleware::validate_signature::RequireValidSignature,
        providers::{Provider, PushProvider},
        state::AppState,
        stores::StoreError,
    },
    axum::{
        extract::{Json, Path, State as StateExtractor},
        http::StatusCode,
    },
    serde::{Deserialize, Serialize},
    std::sync::Arc,
};

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct MessagePayload {
    pub topic: Option<String>,
    pub flags: u32,
    pub blob: String,
}

impl MessagePayload {
    pub fn is_encrypted(&self) -> bool {
        (self.flags & ENCRYPTED_FLAG) == ENCRYPTED_FLAG
    }
}

#[derive(Serialize, Deserialize)]
pub struct PushMessageBody {
    pub id: String,
    pub payload: MessagePayload,
}

pub async fn handler(
    Path((tenant_id, id)): Path<(String, String)>,
    StateExtractor(state): StateExtractor<Arc<AppState>>,
    RequireValidSignature(Json(body)): RequireValidSignature<Json<PushMessageBody>>,
) -> Result<Response> {
    increment_counter!(state.metrics, received_notifications);

    let id = id
        .trim_start_matches(DECENTRALIZED_IDENTIFIER_PREFIX)
        .to_string();

    let client = match state.client_store.get_client(&tenant_id, &id).await {
        Ok(c) => Ok(c),
        Err(StoreError::NotFound(_, _)) => Err(ClientNotFound),
        Err(e) => Err(Store(e)),
    }?;
    info!("fetched client ({}) for tenant ({})", &id, &tenant_id);

    if let Ok(_notification) = state
        .notification_store
        .get_notification(&body.id, &tenant_id)
        .await
    {
        info!(
            "notification ({}) already received for client ({})",
            body.id, id
        );
        return Ok(Response::new_success(StatusCode::OK));
    }

    let notification = state
        .notification_store
        .create_or_update_notification(&body.id, &tenant_id, &id, &body.payload)
        .await?;
    info!(
        "stored notification ({}) for tenant ({})",
        &notification.id, &tenant_id
    );

    // TODO make better by only ignoring if previously executed successfully
    // If notification received more than once then discard
    if notification.previous_payloads.len() > 1 {
        info!(
            "notification ({}) already received for client ({})",
            body.id, id
        );
        return Ok(Response::new_success(StatusCode::OK));
    }

    let tenant = state.tenant_store.get_tenant(&tenant_id).await?;
    info!(
        "fetched tenant ({}) during notification ({})",
        &tenant_id, &notification.id
    );

    let mut provider = tenant.provider(&client.push_type)?;
    info!(
        "fetched provider ({}) for tenant ({}) during notification ({})",
        client.push_type.as_str(),
        &tenant_id,
        &notification.id
    );

    provider
        .send_notification(client.token, body.payload)
        .await?;
    info!(
        "sent notification to provider ({}) for tenant ({}) during notification ({})",
        client.push_type.as_str(),
        &tenant_id,
        &notification.id
    );

    // Provider specific metrics
    match provider {
        Provider::Fcm(_) => increment_counter!(state.metrics, sent_fcm_notifications),
        Provider::Apns(_) => increment_counter!(state.metrics, sent_apns_notifications),
        Provider::Noop(_) => {}
    }

    Ok(Response::new_success(StatusCode::ACCEPTED))
}
