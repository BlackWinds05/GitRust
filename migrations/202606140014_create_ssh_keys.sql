CREATE TABLE ssh_keys (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    title VARCHAR(256) NOT NULL,
    public_key TEXT NOT NULL,
    fingerprint VARCHAR(128) UNIQUE NOT NULL,
    created_at TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX idx_ssh_keys_user ON ssh_keys(user_id);
