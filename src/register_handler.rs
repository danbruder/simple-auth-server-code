use actix::{Handler, Message};
use chrono::Local;
use diesel::prelude::*;
use uuid::Uuid;

use crate::errors::ServiceError;
use crate::models::{DbExecutor, Invitation, SlimUser, User};
use crate::utils::hash_password;

#[derive(Debug, Deserialize)]
pub struct UserData {
    pub password: String,
}

// to be used to send data via the Actix actor system
#[derive(Debug)]
pub struct RegisterUser {
    pub invitation_id: String,
    pub password: String,
}

impl Message for RegisterUser {
    type Result = Result<SlimUser, ServiceError>;
}

impl Handler<RegisterUser> for DbExecutor {
    type Result = Result<SlimUser, ServiceError>;

    fn handle(&mut self, msg: RegisterUser, _: &mut Self::Context) -> Self::Result {
        use crate::schema::invitations::dsl::{id, invitations};
        use crate::schema::users::dsl::users;
        let conn: &PgConnection = &self.0.get().unwrap();
        // try parsing the string provided by the user as url parameter
        // return early with error that will be converted to ServiceError
        let invitation_id = Uuid::parse_str(&msg.invitation_id)?;

        invitations
            .filter(id.eq(invitation_id))
            .load::<Invitation>(conn)
            .map_err(|_| ServiceError::BadRequest("Invalid Invitation".into()))
            .and_then(|mut result| {
                if let Some(invitation) = result.pop() {
                    if invitation.expires_at > Local::now().naive_local() {
                        let password: String = hash_password(&msg.password)?;
                        let user = User::from_details(invitation.email, password);
                        let inserted_user: User =
                            diesel::insert_into(users).values(&user).get_result(conn)?;

                        return Ok(inserted_user.into());
                    }
                }

                Err(ServiceError::BadRequest("Invalid Invitation".into()))
            })
    }
}
