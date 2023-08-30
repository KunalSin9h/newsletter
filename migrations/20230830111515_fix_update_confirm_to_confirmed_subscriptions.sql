BEGIN;
    UPDATE subscriptions
        SET status = 'confirmed'
        WHERE status = 'confirm';

    ALTER TABLE subscriptions ALTER COLUMN status SET NOT NULL;
COMMIT;