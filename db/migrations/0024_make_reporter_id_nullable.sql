-- Allow anonymous reporting without authentication
ALTER TABLE reports ALTER COLUMN reporter_id DROP NOT NULL;
