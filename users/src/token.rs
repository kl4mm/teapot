use std::collections::BTreeMap;

use dblib::users::users::User;
use hmac::{Hmac, Mac};
use hyper::http::HeaderValue;
use jwt::{SignWithKey, VerifyWithKey};
use sha2::Sha256;

const KEY: &[u8] = b"secret";

#[derive(Debug)]
pub enum TokenError {
    Hmac,
    Sign,
    Verify,
}

pub fn gen_token(user: &User) -> Result<HeaderValue, TokenError> {
    let key: Hmac<Sha256> = Hmac::new_from_slice(KEY).map_err(|_e| TokenError::Hmac)?;

    let mut claims: BTreeMap<&str, String> = BTreeMap::new();
    claims.insert("id", user.id.to_string());
    claims.insert("email", user.email.to_owned());
    claims.insert("role", "user".to_owned());

    let token = claims
        .sign_with_key(&key)
        .map(|t| HeaderValue::from_str(&t).unwrap())
        .map_err(|_| TokenError::Sign)?;

    Ok(token)
}

pub fn verify_token(token: &str) -> Result<BTreeMap<String, String>, TokenError> {
    let key: Hmac<Sha256> = Hmac::new_from_slice(KEY).map_err(|_e| TokenError::Hmac)?;

    let token: BTreeMap<String, String> = token
        .verify_with_key(&key)
        .map_err(|_| TokenError::Verify)?;

    Ok(token)
}
