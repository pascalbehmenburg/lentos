pub(crate) mod pg_auth;
pub(crate) mod pg_user_repo;

use shared::models::user::{CreateUser, UpdateUser};
use crate::routes::user::User;
use crate::error::Result;

pub(crate) trait UserRepository: Send + Sync + 'static {
    async fn get_by_email(&self, email: &str) -> Result<User>;

    async fn get_by_id(&self, id: &i64) -> Result<User>;

    async fn create(&self, user: &CreateUser) -> Result<()>;

    async fn update(&self, user: &UpdateUser, id: &i64) -> Result<()>;

    async fn delete(&self, id: &i64) -> Result<()>;
}
