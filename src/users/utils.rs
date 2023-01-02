use axum::{
    extract::TypedHeader,
    headers::{authorization::Bearer, Authorization},
    http::request::Parts,
    RequestPartsExt,
};
use hmac::{Hmac, Mac};
use jwt::VerifyWithKey;
use md5::{Digest, Md5};
use rand::Rng;
use sha2::Sha256;

use super::errors::AuthError;
use super::models::{TokenData, User, UserLogin};
use crate::AppState;

const ITOA64: [&str; 64] = [
    ".", "/", "0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "A", "B", "C", "D", "E", "F", "G",
    "H", "I", "J", "K", "L", "M", "N", "O", "P", "Q", "R", "S", "T", "U", "V", "W", "X", "Y", "Z",
    "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p", "q", "r", "s",
    "t", "u", "v", "w", "x", "y", "z",
];

fn encode64(input: &[u8], count: usize) -> String {
    let mut output = String::from("");
    let mut i = 0;

    while i < count {
        let mut value = input[i] as u32;
        i += 1;
        output += ITOA64[(value & 0x3F) as usize];
        if i < count {
            value |= (input[i] as u32) << 8;
        }
        output += ITOA64[((value >> 6) & 0x3F) as usize];
        if i >= count {
            break;
        }
        i += 1;
        if i < count {
            value |= (input[i] as u32) << 16;
        }
        output += ITOA64[((value >> 12) & 0x3F) as usize];
        if i >= count {
            break;
        }
        i += 1;
        output += ITOA64[((value >> 18) & 0x3F) as usize]
    }

    output
}

fn get_salt() -> String {
    let seed: [u8; 6] = rand::thread_rng().gen();
    let salt = encode64(&seed, 6);
    salt
}

fn hash_password(password: &str, salt: &str) -> String {
    let password_bytes = password.as_bytes();
    let salt_bytes = salt.as_bytes();
    let salt_password = [salt_bytes, password_bytes].concat();

    let mut hasher = Md5::new();
    hasher.update(&salt_password);
    let mut hash = hasher.finalize_reset();

    for _ in 0..8192 {
        let hash_password = [&hash, password_bytes].concat();

        hasher.update(&hash_password);
        hash = hasher.finalize_reset();
    }

    let hash_string = encode64(&hash, 16);
    hash_string
}

fn verify(plain_password: &str, hashed_password: &str) -> bool {
    if hashed_password.len() < 12 {
        return false;
    }
    let salt = &hashed_password[4..12];
    let hash = hash_password(plain_password, &salt);
    return hash == hashed_password[12..];
}

pub fn hash(password: &str) -> String {
    let salt = get_salt();
    let hashed_password = format!("$P$B{}{}", salt, hash_password(password, &salt));
    hashed_password
}

pub async fn authenticate_user(state: &AppState, user_login: &UserLogin) -> Option<User> {
    let user_sql = format!(
        r#"
        SELECT *
        FROM {users_table}
        WHERE {users_table}."mail" = ?1
        "#,
        users_table = &state.users_table
    );
    if let Ok(user) = sqlx::query_as::<_, User>(&user_sql)
        .bind(&user_login.mail)
        .fetch_one(&state.pool)
        .await
    {
        let user_password = user.password.clone().unwrap_or(String::from(""));
        let valid = verify(&user_login.password, &user_password);
        if valid {
            return Some(user);
        }
    }
    None
}

pub async fn get_user(parts: &mut Parts, state: AppState) -> Result<User, AuthError> {
    let TypedHeader(Authorization(bearer)) = parts
        .extract::<TypedHeader<Authorization<Bearer>>>()
        .await
        .map_err(|_| AuthError::InvalidToken)?;

    let secret_key = state.secret_key;

    let key: Hmac<Sha256> =
        Hmac::new_from_slice(secret_key.as_bytes()).map_err(|_| AuthError::InvalidToken)?;
    let token_data: TokenData = bearer
        .token()
        .verify_with_key(&key)
        .map_err(|_| AuthError::InvalidToken)?;

    let user_id = token_data.sub;
    let user_sql = format!(
        r#"
        SELECT *
        FROM {users_table}
        WHERE {users_table}."uid" == ?1
        ORDER BY {users_table}."uid"
        "#,
        users_table = &state.users_table
    );
    if let Ok(user) = sqlx::query_as::<_, User>(&user_sql)
        .bind(user_id)
        .fetch_one(&state.pool)
        .await
    {
        return Ok(user);
    }
    Err(AuthError::InvalidToken)
}
