use reqwest::Client;

use crate::domain::SubscriberEmail;

#[derive(Clone)]
pub struct EmailClient {
    http_client: Client,
    base_url: reqwest::Url,
    sender: SubscriberEmail,
}

impl EmailClient {
    pub fn new(base_url: String, sender: SubscriberEmail) -> Result<Self, String> {
        let base_url = match reqwest::Url::parse(&base_url) {
            Ok(url) => url,
            Err(err) => return Err(err.to_string()),
        };

        Ok(Self {
            http_client: Client::new(),
            base_url,
            sender,
        })
    }

    pub async fn send_email(
        &self,
        recipient: SubscriberEmail,
        subject: &str,
        html_content: &str,
        text_content: &str,
    ) -> Result<(), String> {
        let url = match self.base_url.join("email") {
            Ok(url) => url,
            Err(err) => return Err(err.to_string()),
        };

        Ok(())
    }
}

#[cfg(test)]

mod tests {

    use super::EmailClient;
    use super::SubscriberEmail;
    use fake::faker::internet::en::SafeEmail;
    use fake::faker::lorem::en::{Paragraph, Sentence};
    use fake::{Fake, Faker};
    use wiremock::matchers::any;
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn send_email_fires_a_request_to_base_url() {
        let mock_server = MockServer::start().await;
        let sender = SubscriberEmail::parse(SafeEmail().fake()).unwrap();
        let email_client =
            EmailClient::new(mock_server.uri(), sender).expect("Failed to parse mock uri");

        Mock::given(any())
            .respond_with(ResponseTemplate::new(200))
            .expect(1) // number of requests
            .mount(&mock_server)
            .await;

        let subscriber_email = SubscriberEmail::parse(SafeEmail().fake()).unwrap();
        let subject: String = Sentence(1..2).fake();
        let content: String = Paragraph(1..10).fake();

        let _ = email_client
            .send_email(subscriber_email, &subject, &content, &content)
            .await;
    }
}
