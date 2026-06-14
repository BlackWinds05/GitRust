CREATE TABLE repositories (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    owner_type VARCHAR(8) NOT NULL CHECK (owner_type IN ('user', 'group')),
    owner_id UUID NOT NULL,
    name VARCHAR(128) NOT NULL,
    description TEXT,
    default_branch VARCHAR(255) DEFAULT 'main',
    is_private BOOLEAN DEFAULT false,
    is_archived BOOLEAN DEFAULT false,
    is_template BOOLEAN DEFAULT false,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE (owner_type, owner_id, name)
);

CREATE INDEX idx_repos_owner ON repositories(owner_type, owner_id);
