use super::subscriber_name::SubscriberName;
use super::subscriber_email::SubscriberEmail;
use super::super::routes::FormData;

pub struct NewSubscriber {
    pub email: SubscriberEmail,
    pub name: SubscriberName
}

impl TryFrom<FormData> for NewSubscriber {
    type Error = String;

    fn try_from(value: FormData) -> Result<Self, Self::Error> {
        let name = SubscriberName::parse(value.name)?;
        let email = SubscriberEmail::parse(value.email)?;

        Ok(Self { name, email })
    }
}
