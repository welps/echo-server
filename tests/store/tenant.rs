use {
    crate::context::StoreContext,
    echo_server::stores::tenant::{
        TenantApnsUpdateAuth,
        TenantApnsUpdateParams,
        TenantFcmUpdateParams,
        TenantUpdateParams,
    },
    test_context::test_context,
    uuid::Uuid,
};

#[test_context(StoreContext)]
#[tokio::test]
async fn test_tenant_creation(ctx: &mut StoreContext) {
    let res = ctx
        .tenants
        .create_tenant(TenantUpdateParams {
            id: Uuid::new_v4().to_string(),
        })
        .await;

    assert!(res.is_ok())
}

#[test_context(StoreContext)]
#[tokio::test]
async fn test_tenant_update(ctx: &mut StoreContext) {
    let tenant = ctx
        .tenants
        .create_tenant(TenantUpdateParams {
            id: Uuid::new_v4().to_string(),
        })
        .await
        .expect("creation failed");

    let res = ctx
        .tenants
        .update_tenant(&tenant.id, TenantUpdateParams {
            id: Uuid::new_v4().to_string(),
        })
        .await;

    assert!(res.is_ok())
}

#[test_context(StoreContext)]
#[tokio::test]
async fn test_tenant_fcm(ctx: &mut StoreContext) {
    let tenant = ctx
        .tenants
        .create_tenant(TenantUpdateParams {
            id: Uuid::new_v4().to_string(),
        })
        .await
        .expect("creation failed");

    let res = ctx
        .tenants
        .update_tenant_fcm(&tenant.id, TenantFcmUpdateParams {
            fcm_api_key: "test-api-key".to_string(),
        })
        .await;

    assert!(res.is_ok())
}

#[test_context(StoreContext)]
#[tokio::test]
async fn test_tenant_apns(ctx: &mut StoreContext) {
    let tenant = ctx
        .tenants
        .create_tenant(TenantUpdateParams {
            id: Uuid::new_v4().to_string(),
        })
        .await
        .expect("creation failed");

    let res = ctx
        .tenants
        .update_tenant_apns(&tenant.id, TenantApnsUpdateParams {
            apns_topic: "com.walletconect.exampleapp".to_string(),
        })
        .await;

    assert!(res.is_ok())
}

#[test_context(StoreContext)]
#[tokio::test]
async fn test_tenant_apns_certificate_auth(ctx: &mut StoreContext) {
    let tenant = ctx
        .tenants
        .create_tenant(TenantUpdateParams {
            id: Uuid::new_v4().to_string(),
        })
        .await
        .expect("creation failed");

    let res = ctx
        .tenants
        .update_tenant_apns_auth(&tenant.id, TenantApnsUpdateAuth::Certificate {
            apns_certificate: "example-certificate-string".to_string(),
            apns_certificate_password: "password123".to_string(),
        })
        .await;

    assert!(res.is_ok())
}

#[test_context(StoreContext)]
#[tokio::test]
async fn test_tenant_apns_token_auth(ctx: &mut StoreContext) {
    let tenant = ctx
        .tenants
        .create_tenant(TenantUpdateParams {
            id: Uuid::new_v4().to_string(),
        })
        .await
        .expect("creation failed");

    let res = ctx
        .tenants
        .update_tenant_apns_auth(&tenant.id, TenantApnsUpdateAuth::Token {
            apns_pkcs8_pem: "example-pem-string".to_string(),
            apns_key_id: "123".to_string(),
            apns_team_id: "456".to_string(),
        })
        .await;

    assert!(res.is_ok())
}
