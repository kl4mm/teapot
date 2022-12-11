use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use serde::Serialize;
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

const SALT: &str = "salt";

#[derive(Serialize, FromRow)]
struct User {
    id: i64,
    first_name: String,
    last_name: String,
    email: String,
}

impl User {
    pub async fn new(
        pool: &PgPool,
        first_name: String,
        last_name: String,
        email: String,
        password: &[u8],
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO users (
                id,
                first_name,
                last_name,
                email,
                password
            ) VALUES ($1, $2, $3, $4, $5)
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(first_name)
        .bind(last_name)
        .bind(email)
        .bind(password)
        .execute(&*pool)
        .await?;
        Ok(())
    }

    pub async fn from_id(pool: &PgPool, id: uuid::Uuid) -> Result<User, sqlx::Error> {
        let row = sqlx::query_as(
            r#"
            SELECT id, first_name, last_name, email FROM accounts
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_one(pool)
        .await?;

        Ok(row)
    }

    pub async fn from_email(pool: &PgPool, email: String) -> Result<User, sqlx::Error> {
        let row = sqlx::query_as(
            r#"
            SELECT id, first_name, last_name, email FROM accounts
            WHERE email = $1
            "#,
        )
        .bind(email)
        .fetch_one(pool)
        .await?;

        Ok(row)
    }
}

pub struct Password;

impl Password {
    pub fn hash(password: &str) -> Result<String, argon2::password_hash::Error> {
        let salt = SaltString::new(SALT)?;
        let argon2 = Argon2::default();
        let hash = argon2.hash_password(password.as_bytes(), &salt).unwrap();

        Ok(hash.hash.unwrap().to_string())
    }

    pub fn verify(password: &str) -> Result<(), argon2::password_hash::Error> {
        let argon2 = Argon2::default();
        let hashed = Self::hash(password)?;
        let hashed = PasswordHash::new(&hashed)?;
        argon2.verify_password(password.as_bytes(), &hashed)
    }
}
