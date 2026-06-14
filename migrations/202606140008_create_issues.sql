CREATE TABLE issues (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    repository_id UUID NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    number INT NOT NULL,
    title VARCHAR(512) NOT NULL,
    description TEXT,
    author_id UUID NOT NULL REFERENCES users(id),
    state VARCHAR(8) DEFAULT 'open' CHECK (state IN ('open', 'closed')),
    milestone_id UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    closed_at TIMESTAMPTZ,
    UNIQUE (repository_id, number)
);

CREATE INDEX idx_issues_repo ON issues(repository_id, number);
