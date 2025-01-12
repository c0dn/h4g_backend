use argon2::{Algorithm, Argon2, Params, PasswordHash, PasswordHasher, PasswordVerifier, Version};
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;
use log::{debug, error};
use pasetors::claims::{Claims, ClaimsValidationRules};
use pasetors::keys::AsymmetricPublicKey;
use pasetors::{public, Public};
use pasetors::token::UntrustedToken;
use pasetors::version4::V4;
use trust_dns_resolver::AsyncResolver;
use trust_dns_resolver::config::{ResolverConfig, ResolverOpts};
use trust_dns_resolver::proto::rr::RecordType;
use crate::paseto::get_private_public_keypair;
use crate::regex;
use crate::req_res::AppError;

pub fn validate_token(token: &str) -> Option<(String, Claims)> {
    let (_, public_key) = get_private_public_keypair();
    let pk = AsymmetricPublicKey::<V4>::try_from(public_key.as_str()).ok()?;
    let validation_rules = ClaimsValidationRules::new();
    let untrusted_token = UntrustedToken::<Public, V4>::try_from(token).ok()?;
    let trusted = public::verify(&pk, &untrusted_token, &validation_rules, None, None).ok()?;
    let claims = trusted.payload_claims()?;
    let role = claims.get_claim("role")?.as_str()?;
    Some((role.to_string(), claims.clone()))
}


pub fn hash_password(password: &str) -> Result<String, AppError> {
    // Reference https://github.com/OWASP/CheatSheetSeries/blob/master/cheatsheets/Password_Storage_Cheat_Sheet.md#argon2id
    let parameters = Params::new(19456, 2, 1, None).unwrap();
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, parameters);
    let salt = SaltString::generate(&mut OsRng);
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|err| {
            error!("Failed to hash password: {}", err);
            AppError::internal_error("Password hashing failed".to_string())
        })?
        .to_string();
    Ok(password_hash)
}

pub fn verify_password(hash: &str, password: &str) -> Result<(), AppError> {
    let parsed_hash = PasswordHash::new(&hash).map_err(|_| AppError::unauthorized())?;
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .map_err(|_| AppError::unauthorized())
}


pub async fn is_bad_mail(email: &str) -> bool {
    let re = regex!(r"^[\w.-]+@[a-zA-Z\d.-]+\.[a-zA-Z]{2,}$");

    if !re.is_match(email) {
        return true;
    }
    let domain = email.split('@').last().unwrap_or("");
    let resolver = AsyncResolver::tokio(ResolverConfig::default(), ResolverOpts::default());
    let (have_valid_mx, mx_base_domains) = match resolver.lookup(domain, RecordType::MX).await {
        Ok(records) => {
            let mx_hostnames = records
                .iter()
                .filter_map(|record| record.as_mx()) // Filter out records which are not MX records
                .map(|record| record.exchange().to_utf8())
                .collect::<Vec<String>>();
            let base_domains = mx_hostnames
                .into_iter()
                .filter_map(|hostname| {
                    let mut parts = hostname.trim_end_matches('.').split('.').rev();
                    let tld = parts.next()?;
                    let domain = parts.next()?;
                    Some(format!("{}.{}", domain, tld))
                })
                .collect::<Vec<String>>();
            (!records.is_empty(), base_domains)
        }
        Err(e) => {
            error!(
                "Error looking up MX records for domain: {}, error: {:?}",
                domain, e
            );
            (false, vec![])
        }
    };
    // Keep adding domains to this list as we find them
    let whitelisted_mx_domains = [
        "gmail.com",
        "yahoo.com",
        "outlook.com",
        "simplelogin.co",
        "yahoodns.net",
        "icloud.com",
        "protonmail.ch",
    ];
    let blacklisted_mx_domain = ["fex.plus"];
    let is_whitelisted_mx = mx_base_domains
        .iter()
        .any(|domain| whitelisted_mx_domains.contains(&domain.as_str()));
    let is_blacklisted_mx = mx_base_domains.iter().any(|domain| {
        blacklisted_mx_domain
            .iter()
            .any(|blacklisted_domain| blacklisted_domain == domain)
    });
    let have_valid_a = match resolver.lookup(domain, RecordType::A).await {
        Ok(records) => !records.is_empty(),
        Err(e) => {
            error!(
                "Error looking up A records for domain: {}, error: {:?}",
                domain, e
            );
            false
        }
    };
    debug!(
        "Domain: {}, have_valid_mx: {}, have_valid_a: {}, mx_base_domains: {:?}, is_whitelisted_mx: {}, is_blacklisted_mx: {}",
        domain, have_valid_mx, have_valid_a, mx_base_domains, is_whitelisted_mx, is_blacklisted_mx
    );
    if is_blacklisted_mx {
        true
    } else if have_valid_mx && have_valid_a {
        false
    } else if have_valid_mx && !have_valid_a {
        !is_whitelisted_mx
    } else {
        true
    }
}


#[macro_export]
macro_rules! regex {
    ($re:literal $(,)?) => {{
        static RE: once_cell::sync::OnceCell<regex::Regex> = once_cell::sync::OnceCell::new();
        RE.get_or_init(|| regex::Regex::new($re).unwrap())
    }};
}