CREATE TABLE repository_members (
    repository_id UUID NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    permission VARCHAR(16) NOT NULL CHECK (permission IN ('read', 'write', 'admin')),
    added_at TIMESTAMPTZ DEFAULT now(),
    PRIMARY KEY (repository_id, user_id)
);
