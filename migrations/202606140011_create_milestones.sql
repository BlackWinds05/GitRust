CREATE TABLE milestones (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    repository_id UUID NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    title VARCHAR(256) NOT NULL,
    description TEXT,
    due_date DATE,
    state VARCHAR(8) DEFAULT 'open' CHECK (state IN ('open', 'closed')),
    created_at TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX idx_milestones_repo ON milestones(repository_id);
