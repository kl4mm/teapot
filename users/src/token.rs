use std::{collections::BTreeMap, env};

use dblib::users::users::User;
use hmac::{Hmac, Mac};
use hyper::http::HeaderValue;
use jwt::{SignWithKey, VerifyWithKey};
use sha2::Sha256;

#[derive(Debug)]
pub enum TokenError {
    Hmac,
    Sign,
    Verify,
}

fn get_key() -> Result<Hmac<Sha256>, TokenError> {
    let key = env::var("TOKEN_SECRET").expect("TOKEN_SECRET must be set");
    Hmac::new_from_slice(key.as_bytes()).map_err(|_e| TokenError::Hmac)
}

pub fn gen_token(user: &User) -> Result<HeaderValue, TokenError> {
    let key = get_key()?;

    let claims: BTreeMap<&str, String> = BTreeMap::from([
        ("id", user.id.to_string()),
        ("email", user.email.to_owned()),
        ("role", "user".to_owned()),
    ]);

    let token = claims
        .sign_with_key(&key)
        .map(|t| HeaderValue::from_str(&t).unwrap())
        .map_err(|_| TokenError::Sign)?;

    Ok(token)
}

pub fn verify_token(token: &str) -> Result<BTreeMap<String, String>, TokenError> {
    let key = get_key()?;

    let token: BTreeMap<String, String> = token
        .verify_with_key(&key)
        .map_err(|_| TokenError::Verify)?;

    Ok(token)
}
