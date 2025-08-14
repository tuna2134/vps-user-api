use std::env;

use lettre::{
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor, message::header::ContentType,
    transport::smtp::authentication::Credentials,
};

pub async fn send_passcode(code: String, mail_to: String) -> anyhow::Result<()> {
    tracing::debug!("Sending passcode email to {}", mail_to);
    let msg = Message::builder()
        .from(env::var("MAIL_FROM")?.parse()?)
        .to(mail_to.parse()?)
        .subject("Your passcode")
        .header(ContentType::TEXT_PLAIN)
        .body(format!("Your passcode is: {}", code))?;

    tracing::debug!("Email message created: {:?}", env::var("SMTP_HOSTNAME")?.as_str());
    let creds = Credentials::new(env::var("SMTP_USERNAME")?, env::var("SMTP_PASSWORD")?);
    let mailer: AsyncSmtpTransport<Tokio1Executor> =
        AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&env::var("SMTP_HOSTNAME")?)?
            .credentials(creds)
            .build();

    mailer.send(msg).await?;
    Ok(())
}
