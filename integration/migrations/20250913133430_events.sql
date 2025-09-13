-- Create events table
CREATE TABLE events (
    id UUID PRIMARY KEY NOT NULL,
    name VARCHAR(255) NOT NULL,
    payload JSONB NOT NULL
);
