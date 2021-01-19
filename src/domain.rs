//! src/domain.rs

mod subscriber_name;

mod subscriber_email;

mod new_subscriber;

pub use new_subscriber::NewSubscriber;
pub use subscriber_email::SubscriberEmail;
pub use subscriber_name::SubscriberName;
