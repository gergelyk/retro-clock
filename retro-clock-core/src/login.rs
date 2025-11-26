use anyhow::Context;
use base64::{engine::general_purpose::STANDARD, Engine as _};
use min_jwt::{sign::ring::HmacKeySigner, verify::ring::HmacKeyVerifier};
use pbkdf2::pbkdf2_hmac;
use ring::hmac;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::time::{SystemTime, UNIX_EPOCH};

pub const JWT_SECRET_BASE64: &str = env!("JWT_SECRET_BASE64");
pub const JWT_SUBJECT: &str = "admin";
pub const HMAC_SALT_BASE64: &str = env!("HMAC_SALT_BASE64");

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    sub: String, // subject (user id, email, etc.)
    exp: usize,  // expiration as unix timestamp
}

pub fn generate_jwt() -> anyhow::Result<String> {
    let expiration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("Failed to get current timestamp")?
        .as_secs()
        + 3600; // 1 hour validity

    let claims = Claims {
        sub: JWT_SUBJECT.to_string(),
        exp: expiration as usize,
    };

    let header = String::from(r#"{"typ":"JWT","alg":"HS256"}"#);
    let claims_string = serde_json::to_string(&claims).context("Failed to encode JWT claims")?;

    let jwt_secret = STANDARD.decode(JWT_SECRET_BASE64)?;
    let key = hmac::Key::new(hmac::HMAC_SHA256, &jwt_secret);
    let signer = HmacKeySigner::with_hs256(&key);
    let token = min_jwt::encode_and_sign(header.as_bytes(), claims_string.as_bytes(), signer)
        .context("Failed to sign JWT")?;

    Ok(token)
}

pub fn verify_jwt(token: &str) -> anyhow::Result<Claims> {
    let jwt_secret = STANDARD.decode(JWT_SECRET_BASE64)?;
    let hmac_key = hmac::Key::new(hmac::HMAC_SHA256, &jwt_secret);
    let verifier = HmacKeyVerifier::with_hs256(&hmac_key);
    let signature_verified_jwt =
        min_jwt::verify(token, &verifier).context("Invalid JWT signature")?;
    let claims_vector = signature_verified_jwt
        .decode_claims()
        .context("Failed to decode JWT claims to vector")?;
    let claims_string =
        String::from_utf8(claims_vector).context("Failed to convert JWT claims to string")?;
    let claims: Claims =
        serde_json::from_str(&claims_string).context("Failed to decode JWT claims")?;

    let now = SystemTime::now();
    let timestamp = now
        .duration_since(UNIX_EPOCH)
        .context("Failed to get current timestamp")?
        .as_secs() as usize;
    if claims.exp < timestamp {
        anyhow::bail!("JWT token expired");
    }
    if claims.sub != JWT_SUBJECT {
        anyhow::bail!("Invalid JWT subject");
    }
    Ok(claims)
}

pub fn hash_password(password: &str) -> anyhow::Result<String> {
    // Our secret salt, "esp32" would be "ZXNwMzI="
    // You can test it versus https://8gwifi.org/pbkdf.jsp
    // Use dkLen=256
    let salt = STANDARD.decode(HMAC_SALT_BASE64)?;
    let rounds = 1000;
    let mut hash = [0u8; 32];
    pbkdf2_hmac::<Sha256>(password.as_bytes(), &salt, rounds, &mut hash);
    let hash_b64 = STANDARD.encode(hash);
    Ok(hash_b64)
}

pub fn verify_password(password: &str, expected_hash_b64: &str) -> anyhow::Result<bool> {
    let hash_b64 = hash_password(password)?;
    let is_valid = hash_b64 == expected_hash_b64;
    Ok(is_valid)
}
