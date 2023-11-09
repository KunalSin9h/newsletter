CREATE TABLE newsletter_issue (
    newsletter_issue_id uuid NOT NULL,
    title TEXT NOT NULL,
    text TEXT NOT NULL,
    html TEXT NOT NULL,
    published_at TIMESTAMPTZ NOT NULL,
    PRIMARY KEY (newsletter_issue_id)
);
