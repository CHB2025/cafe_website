use std::{error::Error, fmt::Display};

use crate::config::config;

use super::EmailKind;
use lettre::{
    address::AddressError,
    message::{Mailbox, SinglePart},
    Address, AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};
use sqlx::FromRow;
use tracing::debug;
use uuid::Uuid;

#[derive(FromRow, Clone)]
struct EmailToSend {
    id: Uuid,
    kind: EmailKind,
    subject: String,
    message: String,
    to: String,
}

enum EmailError {
    Address(AddressError),
    Email(lettre::error::Error),
    Smtp(lettre::transport::smtp::Error),
}

impl Display for EmailError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Address(a) => a.to_string(),
            Self::Email(e) => e.to_string(),
            Self::Smtp(s) => s.to_string(),
        };
        write!(f, "{}", s)
    }
}

impl From<AddressError> for EmailError {
    fn from(value: AddressError) -> Self {
        Self::Address(value)
    }
}
impl From<lettre::error::Error> for EmailError {
    fn from(value: lettre::error::Error) -> Self {
        Self::Email(value)
    }
}
impl From<lettre::transport::smtp::Error> for EmailError {
    fn from(value: lettre::transport::smtp::Error) -> Self {
        Self::Smtp(value)
    }
}

pub async fn send_all() -> Result<(), Box<dyn Error>> {
    let emails = sqlx::query_as!(
        EmailToSend,
        r#"SELECT 
            id, 
            kind AS "kind: _", 
            subject, 
            message, 
            address as to
        FROM email
        WHERE status = 'pending'"#
    )
    .fetch_all(config().pool())
    .await?;
    if emails.is_empty() {
        return Ok(());
    }
    debug!("Sending {} Emails", emails.len());

    let transport = config().mailer().expect("Email system not set up");
    let address = config().mailing_address().expect("Emailing not set up");
    let mailbox = Mailbox::new(None, address.clone());
    for email in emails {
        let this_id = email.id;
        let res = try_build(email, mailbox.clone());
        if let Ok(msg) = res {
            if let Err(e) = try_send(msg, transport).await {
                // Mark email as failed
                tracing::error!("Failed to send email: {}", e);
                sqlx::query!(
                    "UPDATE email SET status = 'failed', sent_at = now() WHERE id = $1",
                    this_id
                )
                .execute(config().pool())
                .await?; // Should probably not exit out at this point?
            } else {
                // Mark email as sent
                sqlx::query!(
                    "UPDATE email SET status = 'sent', sent_at = now() WHERE id = $1",
                    this_id
                )
                .execute(config().pool())
                .await?; // Should probably not exit out at this point?
            };
        } else {
            // Mark email as failed
            sqlx::query!(
                "UPDATE email SET status = 'failed', sent_at = now() WHERE id = $1",
                this_id
            )
            .execute(config().pool())
            .await?; // Should probably not exit out at this point?
        }
    }
    Ok(())
}

fn try_build(email: EmailToSend, mbox: Mailbox) -> Result<Message, EmailError> {
    let body = match email.kind {
        EmailKind::Html => SinglePart::html(email.message),
        EmailKind::Text => SinglePart::plain(email.message),
    };
    Ok(Message::builder()
        .subject(email.subject)
        .from(mbox)
        .to(Mailbox::new(None, Address::try_from(email.to)?))
        .singlepart(body)?)
}

async fn try_send(
    msg: Message,
    transport: &AsyncSmtpTransport<Tokio1Executor>,
) -> Result<(), EmailError> {
    transport.send(msg).await?;
    Ok(())
}
