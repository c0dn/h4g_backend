use crate::models::user::AccountType;
use chrono::Utc;
use once_cell::sync::OnceCell;
use pasetors::claims::Claims;
use pasetors::keys::AsymmetricSecretKey;
use pasetors::public;
use pasetors::version4::V4;
use std::fs;
use std::str::FromStr;
use uuid::Uuid;

pub fn get_private_public_keypair() -> (&'static String, &'static String) {
    static SECRET_KEY: OnceCell<String> = OnceCell::new();
    static PUBLIC_KEY: OnceCell<String> = OnceCell::new();
    let s = SECRET_KEY
        .get_or_init(|| fs::read_to_string("web_key.pem").expect("Failed to read secret key"));
    let p = PUBLIC_KEY
        .get_or_init(|| fs::read_to_string("web_public.pem").expect("Failed to read public key"));
    (s, p)
}

pub fn generate_access_token(uuid: &str, role: &str) -> String {
    let (secret_key, _) = get_private_public_keypair();
    let sk = AsymmetricSecretKey::<V4>::try_from(secret_key.as_str()).unwrap();
    let mut claims = Claims::new().unwrap();
    let now = Utc::now();
    let expiration_time = now + chrono::Duration::try_minutes(5).unwrap();
    let expiration_string = expiration_time.to_rfc3339();

    claims.add_additional("user_uid", uuid.to_string()).unwrap();

    claims.add_additional("role", role.to_string()).unwrap();
    claims.expiration(&expiration_string).unwrap();
    public::sign(&sk, &claims, None, None).unwrap()
}

pub fn generate_refresh_token(uuid: &str, role: &str) -> String {
    let (secret_key, _) = get_private_public_keypair();
    let sk = AsymmetricSecretKey::<V4>::try_from(secret_key.as_str()).unwrap();
    let now = Utc::now();
    let expiration_time = now + chrono::Duration::try_days(14).unwrap();
    let expiration_string = expiration_time.to_rfc3339();
    let mut claims = Claims::new().unwrap();
    claims.add_additional("user_uid", uuid.to_string()).unwrap();
    claims.add_additional("role", role.to_string()).unwrap();
    claims.expiration(&expiration_string).unwrap();

    public::sign(&sk, &claims, None, None).unwrap()
}

#[derive(Debug, Clone)]
pub struct AuthTokenClaims {
    pub user_uid: Uuid,
    pub role: AccountType,
}

impl TryFrom<&Claims> for AuthTokenClaims {
    type Error = String;

    fn try_from(claims: &Claims) -> Result<Self, Self::Error> {
        let user_uid = claims
            .get_claim("user_uid")
            .ok_or_else(|| "Unable to deserialize 'username'".to_string())?
            .as_str()
            .ok_or_else(|| "Expected a string for 'username'".to_string())?;

        let user_uid = Uuid::from_str(user_uid).map_err(|e| e.to_string())?;

        let role_str = claims
            .get_claim("role")
            .ok_or_else(|| "Unable to deserialize 'role'".to_string())?
            .as_str()
            .ok_or_else(|| "Expected a string for 'role'".to_string())?;

        let role = AccountType::try_from(role_str)?;

        Ok(Self { user_uid, role })
    }
}
