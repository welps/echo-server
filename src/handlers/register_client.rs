use {
    crate::{
        error::{
            Error::{EmptyField, ProviderNotAvailable},
            Result,
        },
        handlers::{Response, DECENTRALIZED_IDENTIFIER_PREFIX},
        increment_counter,
        log::prelude::*,
        state::AppState,
        stores::client::Client,
    },
    axum::extract::{Json, Path, State as StateExtractor},
    serde::{Deserialize, Serialize},
    std::sync::Arc,
};

#[derive(Serialize, Deserialize)]
pub struct RegisterBody {
    pub client_id: String,
    #[serde(rename = "type")]
    pub push_type: String,
    pub token: String,
}

pub async fn handler(
    Path(tenant_id): Path<String>,
    StateExtractor(state): StateExtractor<Arc<AppState>>,
    Json(body): Json<RegisterBody>,
) -> Result<Response> {
    let push_type = body.push_type.as_str().try_into()?;
    let tenant = state.tenant_store.get_tenant(&tenant_id).await?;
    let supported_providers = tenant.providers();
    if !supported_providers.contains(&push_type) {
        return Err(ProviderNotAvailable(push_type.into()));
    }

    if body.token.is_empty() {
        return Err(EmptyField("token".to_string()));
    }

    let client_id = body
        .client_id
        .trim_start_matches(DECENTRALIZED_IDENTIFIER_PREFIX);

    state
        .client_store
        .create_client(&tenant_id, client_id, Client {
            push_type,
            token: body.token,
        })
        .await?;

    info!(
        "client registered for tenant ({}) using {}",
        tenant_id, body.push_type
    );

    increment_counter!(state.metrics, registered_clients);

    Ok(Response::default())
}
