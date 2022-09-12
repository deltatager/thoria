use std::sync::Arc;

use async_trait::async_trait;
use serenity::Error;
use serenity::model::user::User;
use songbird::Call;
use songbird::typemap::TypeMapKey;
use tokio::sync::Mutex;

use crate::Data;

pub type Context<'a> = poise::Context<'a, Data, Error>;

#[async_trait]
pub trait GetManagerTrait {
    async fn get_handler(&self) -> Option<Arc<Mutex<Call>>>;
}

#[async_trait]
impl <'a, U, E>GetManagerTrait for poise::Context<'a, U, E>
    where U: Sync {

    async fn get_handler(&self) -> Option<Arc<Mutex<Call>>> {
        songbird::get(self.discord())
            .await
            .unwrap()
            .clone()
            .get(self.guild().unwrap().id)
    }
}

pub struct UserKey;

impl TypeMapKey for UserKey {
    type Value = User;
}