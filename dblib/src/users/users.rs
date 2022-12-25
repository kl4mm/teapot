use argon2::{
    password_hash::{PasswordHasher, SaltString},
    Argon2,
};
use serde::Serialize;
use sqlx::{FromRow, PgPool, Row};

const SALT: &str = "c3VwZXJzZWNyZXRzYWx0";

#[derive(Serialize, FromRow)]
pub struct User {
    pub id: i64,
    #[serde(rename(serialize = "firstName"))]
    first_name: String,
    #[serde(rename(serialize = "lastName"))]
    last_name: String,
    pub email: String,
}

impl User {
    pub async fn new(
        pool: &PgPool,
        first_name: String,
        last_name: String,
        email: String,
        password: String,
    ) -> Result<Self, sqlx::Error> {
        let row = sqlx::query(
            r#"
            INSERT INTO users (
                first_name,
                last_name,
                email,
                password
            ) VALUES ($1, $2, $3, $4)
            RETURNING id 
            "#,
        )
        .bind(&first_name)
        .bind(&last_name)
        .bind(&email)
        .bind(password.into_bytes())
        .fetch_one(pool)
        .await?;

        let id = row.try_get("id")?;

        Ok(User {
            id,
            first_name,
            last_name,
            email,
        })
    }

    pub async fn from_email_and_password(
        pool: &PgPool,
        email: String,
        password: &[u8],
    ) -> Result<User, sqlx::Error> {
        let row = sqlx::query_as(
            r#"
            SELECT id, first_name, last_name, email FROM users
            WHERE email = $1 AND password = $2
            "#,
        )
        .bind(email)
        .bind(password)
        .fetch_one(pool)
        .await?;

        Ok(row)
    }

    pub async fn from_id(pool: &PgPool, id: uuid::Uuid) -> Result<User, sqlx::Error> {
        let row = sqlx::query_as(
            r#"
            SELECT id, first_name, last_name, email FROM users
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_one(pool)
        .await?;

        Ok(row)
    }

    pub async fn from_email(pool: &PgPool, email: &str) -> Result<User, sqlx::Error> {
        let row = sqlx::query_as(
            r#"
            SELECT id, first_name, last_name, email FROM users
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

    // pub fn verify(password: &[u8], stored: &[u8]) -> Result<(), argon2::password_hash::Error> {
    //     argon2.verify_password(password, &stored)
    // }
}
