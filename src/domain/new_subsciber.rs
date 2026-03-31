use crate::domain::{UserEmail, UserName};

pub struct NewSubscriber {
    pub email: UserEmail,
    pub name: UserName,
}
