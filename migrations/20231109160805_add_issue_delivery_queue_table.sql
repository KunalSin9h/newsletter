CREATE TABLE issue_delivery_queue (
    newsletter_issue_id uuid NOT NULL REFERENCES newsletter_issue(newsletter_issue_id) ON DELETE CASCADE,
    subscriber_email TEXT NOT NULL,
    PRIMARY KEY (newsletter_issue_id, subscriber_email)
);
