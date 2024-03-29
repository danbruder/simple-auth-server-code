// models.rs

use crate::schema::{invitations, users};
use actix::{Actor, SyncContext};
use chrono::{Local, NaiveDateTime};
use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool};
use std::convert::From;
use uuid::Uuid;

pub struct DbExecutor(pub Pool<ConnectionManager<PgConnection>>);

impl Actor for DbExecutor {
    type Context = SyncContext<Self>;
}

//------------------------------------------------------------------------------
// User
//------------------------------------------------------------------------------
#[derive(Debug, Serialize, Deserialize, Queryable, Insertable)]
#[table_name = "users"]
pub struct User {
    pub email: String,
    pub password: String,
    pub created_at: NaiveDateTime, // only NaiveDateTime works here due to diesel limitations
}

impl User {
    pub fn from_details(email: String, password: String) -> Self {
        User {
            email,
            password,
            created_at: Local::now().naive_local(),
        }
    }
}

//------------------------------------------------------------------------------
// Slim User
//------------------------------------------------------------------------------
#[derive(Debug, Serialize, Deserialize)]
pub struct SlimUser {
    pub email: String,
}

impl From<User> for SlimUser {
    fn from(user: User) -> Self {
        SlimUser { email: user.email }
    }
}

//------------------------------------------------------------------------------
// Invitation
//------------------------------------------------------------------------------
#[derive(Debug, Serialize, Deserialize, Queryable, Insertable)]
#[table_name = "invitations"]
pub struct Invitation {
    pub id: Uuid,
    pub email: String,
    pub expires_at: NaiveDateTime,
}
