BEGIN;
    UPDATE subscriptions
        SET status = 'confirm'
        WHERE status is NULL;

    ALTER TABLE subscriptions ALTER COLUMN status SET NOT NULL;
COMMIT;