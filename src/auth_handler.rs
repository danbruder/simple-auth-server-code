//auth_handler.rs
use actix::{Handler, Message};
use actix_identity::Identity;
use actix_web::{dev::Payload, Error, FromRequest, HttpRequest};
use bcrypt::verify;
use diesel::prelude::*;

use crate::errors::ServiceError;
use crate::models::{DbExecutor, SlimUser, User};
use crate::utils::decode_token;

#[derive(Debug, Deserialize)]
pub struct AuthData {
    pub email: String,
    pub password: String,
}

impl Message for AuthData {
    type Result = Result<SlimUser, ServiceError>;
}

impl Handler<AuthData> for DbExecutor {
    type Result = Result<SlimUser, ServiceError>;

    fn handle(&mut self, msg: AuthData, _: &mut Self::Context) -> Self::Result {
        use crate::schema::users::dsl::{email, users};
        let conn: &PgConnection = &self.0.get().unwrap();

        let mut items = users.filter(email.eq(&msg.email)).load::<User>(conn)?;

        if let Some(user) = items.pop() {
            if let Ok(matching) = verify(&msg.password, &user.password) {
                if matching {
                    return Ok(user.into());
                }
            }
        }

        Err(ServiceError::BadRequest(
            "Username and password don't match".into(),
        ))
    }
}

pub type LoggedUser = SlimUser;

impl FromRequest for LoggedUser {
    type Error = Error;
    type Future = Result<LoggedUser, Error>;
    type Config = ();

    fn from_request(req: &HttpRequest, pl: &mut Payload) -> Self::Future {
        if let Some(identity) = Identity::from_request(req, pl)?.identity() {
            let user: SlimUser = decode_token(&identity)?;
            return Ok(user as LoggedUser);
        }
        Err(ServiceError::Unauthorized.into())
    }
}
