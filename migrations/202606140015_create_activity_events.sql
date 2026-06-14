CREATE TABLE activity_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id),
    event_type VARCHAR(32) NOT NULL,
    repository_id UUID REFERENCES repositories(id) ON DELETE SET NULL,
    target_type VARCHAR(32),
    target_id UUID,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX idx_activity_user_time ON activity_events(user_id, created_at DESC);
CREATE INDEX idx_activity_repo_time ON activity_events(repository_id, created_at DESC);
