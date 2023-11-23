use crate::handler::api_client::ApiHandler;
use shared::models::user::{CreateUser, SignInUser};

pub(crate) async fn sign_in(
    api_handler: &ApiHandler,
    sign_in_user: SignInUser,
) {
    tracing::debug!("Trying to sign in with provided data...");
    let base_url = api_handler.base_url;

    let sign_in_response = api_handler
        .client
        .post(format!("{base_url}/users/login"))
        .json(&sign_in_user)
        .send()
        .await
        .unwrap();

    if !sign_in_response.status().is_success() {
        tracing::error!(
            "Sign in failed. Server responded with: {:?}",
            sign_in_response
        );
        return;
    }

    tracing::debug!(
        "Signed in successfully. Server responded: {:?}",
        sign_in_response
    );

    api_handler.cookie_store.save();
}

pub(crate) async fn sign_up(
    api_handler: &ApiHandler,
    sign_up_user: CreateUser,
) {
    tracing::debug!("Processing sign up event...");
    let base_url = api_handler.base_url;

    let sign_up_response = &api_handler
        .client
        .post(format!("{base_url}/users/register"))
        .json(&sign_up_user)
        .send()
        .await
        .unwrap();

    if !sign_up_response.status().is_success() {
        tracing::error!(
            "Sign up failed. Server responded with: {:?}",
            sign_up_response
        );
        return;
    }

    tracing::debug!(
        "Signed up successfully. Server responded: {:?}",
        sign_up_response
    );

    tracing::debug!(
        "Logging in user in consequence of successful registration..."
    );
    let sign_in_user = SignInUser {
        email: sign_up_user.email,
        password: sign_up_user.password,
    };

    sign_in(&api_handler, sign_in_user).await;
}
