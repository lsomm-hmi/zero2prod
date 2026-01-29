-- We wrap the whole migration in a transaction to make sure
-- it succeeds or fails atomically. 
BEGIN;
    -- Backfill `status`
    UPDATE subscriptions
        SET status = 'confirmed'
        WHERE status IS NULL;

    -- Make `status` mandatory
    ALTER TABLE subscriptions ALTER COLUMN status SET NOT NULL;
COMMIT;