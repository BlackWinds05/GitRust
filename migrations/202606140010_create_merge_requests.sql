CREATE TABLE merge_requests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    repository_id UUID NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    number INT NOT NULL,
    title VARCHAR(512) NOT NULL,
    description TEXT,
    author_id UUID NOT NULL REFERENCES users(id),
    source_branch VARCHAR(255) NOT NULL,
    target_branch VARCHAR(255) NOT NULL,
    state VARCHAR(8) DEFAULT 'open' CHECK (state IN ('open', 'merged', 'closed')),
    merge_status VARCHAR(16) CHECK (merge_status IN ('can_be_merged', 'conflict', 'blocked')),
    merge_commit_sha VARCHAR(64),
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    merged_at TIMESTAMPTZ,
    merged_by UUID REFERENCES users(id),
    closed_at TIMESTAMPTZ,
    UNIQUE (repository_id, number)
);

CREATE INDEX idx_mrs_repo ON merge_requests(repository_id, number);
