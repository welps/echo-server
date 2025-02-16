use {
    crate::{
        error,
        error::{
            Error,
            Error::{InvalidConfiguration, NoApnsConfigured},
        },
        providers::ProviderKind,
        stores::tenant::ApnsType,
    },
    serde::Deserialize,
};

#[derive(Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct Config {
    #[serde(default = "default_port")]
    pub port: u16,
    pub public_url: String,
    #[serde(default = "default_log_level")]
    pub log_level: String,
    #[serde(default = "default_log_level_otel")]
    pub log_level_otel: String,
    #[serde(default = "default_disable_header")]
    pub disable_header: bool,
    #[serde(default = "default_relay_url")]
    pub relay_url: String,
    #[serde(default = "default_validate_signatures")]
    pub validate_signatures: bool,
    pub database_url: String,
    pub tenant_database_url: Option<String>,
    #[serde(default = "default_tenant_id")]
    pub default_tenant_id: String,
    #[serde(default = "default_is_test", skip)]
    /// This is an internal flag to disable logging, cannot be defined by user
    pub is_test: bool,

    // CORS
    #[serde(default = "default_cors_allowed_origins")]
    pub cors_allowed_origins: Vec<String>,

    // TELEMETRY
    pub otel_exporter_otlp_endpoint: Option<String>,
    pub telemetry_prometheus_port: Option<u16>,

    // APNS
    pub apns_type: Option<ApnsType>,
    pub apns_topic: Option<String>,

    pub apns_certificate: Option<String>,
    pub apns_certificate_password: Option<String>,

    pub apns_pkcs8_pem: Option<String>,
    pub apns_key_id: Option<String>,
    pub apns_team_id: Option<String>,

    // FCM
    pub fcm_api_key: Option<String>,
}

impl Config {
    /// Run validations against config and throw error
    pub fn is_valid(&self) -> error::Result<()> {
        if self.tenant_database_url.is_none() && self.single_tenant_supported_providers().is_empty()
        {
            return Err(InvalidConfiguration(
                "no tenant database url provided and no provider keys found".to_string(),
            ));
        }

        if !self.single_tenant_supported_providers().is_empty()
            && self.tenant_database_url.is_some()
        {
            return Err(InvalidConfiguration(
                "tenant database and providers keys found in config".to_string(),
            ));
        }

        if let Some(tenant_database_url) = &self.tenant_database_url {
            if tenant_database_url == &self.database_url {
                return Err(InvalidConfiguration(
                    "`TENANT_DATABASE_URL` is equal to `DATABASE_URL`, this is not allowed"
                        .to_string(),
                ));
            }
        }

        // Check that APNS config is valid when it has been configured
        match self.get_apns_type() {
            Ok(_) => Ok(()),
            Err(NoApnsConfigured) => Ok(()),
            Err(e) => Err(e),
        }?;

        Ok(())
    }

    pub fn single_tenant_supported_providers(&self) -> Vec<ProviderKind> {
        let mut supported = vec![];

        if self.get_apns_type().is_ok() {
            supported.push(ProviderKind::Apns);
            supported.push(ProviderKind::ApnsSandbox);
        }

        if self.fcm_api_key.is_some() {
            supported.push(ProviderKind::Fcm);
        }

        // Only available in debug/testing
        #[cfg(any(debug_assertions, test))]
        if self.tenant_database_url.is_none() {
            supported.push(ProviderKind::Noop);
        }

        supported
    }

    pub fn get_apns_type(&self) -> Result<ApnsType, Error> {
        if let Some(apns_type) = &self.apns_type {
            // Check if APNS config is correct
            let _ = match apns_type {
                ApnsType::Certificate => match (
                    &self.apns_topic,
                    &self.apns_certificate,
                    &self.apns_certificate_password,
                ) {
                    (Some(_), Some(_), Some(_)) => Ok(ApnsType::Certificate),
                    _ => Err(InvalidConfiguration(
                        "APNS_TYPE of Certificate requires specific variables, please check the \
                         documentation."
                            .to_string(),
                    )),
                },
                ApnsType::Token => match (
                    &self.apns_topic,
                    &self.apns_pkcs8_pem,
                    &self.apns_key_id,
                    &self.apns_team_id,
                ) {
                    (Some(_), Some(_), Some(_), Some(_)) => Ok(ApnsType::Token),
                    _ => Err(InvalidConfiguration(
                        "APNS_TYPE of Certificate requires specific variables, please check the \
                         documentation."
                            .to_string(),
                    )),
                },
            }?;
        }

        Err(NoApnsConfigured)
    }
}

fn default_port() -> u16 {
    3000
}

fn default_log_level() -> String {
    "info,echo-server=info".to_string()
}

fn default_log_level_otel() -> String {
    "info,echo-server=trace".to_string()
}

fn default_disable_header() -> bool {
    false
}

fn default_validate_signatures() -> bool {
    true
}

fn default_relay_url() -> String {
    "https://relay.walletconnect.com".to_string()
}

fn default_tenant_id() -> String {
    "0000-0000-0000-0000".to_string()
}

fn default_is_test() -> bool {
    false
}

fn default_cors_allowed_origins() -> Vec<String> {
    vec!["*".to_string()]
}

pub fn get_config() -> error::Result<Config> {
    let config = envy::from_env::<Config>()?;
    Ok(config)
}
