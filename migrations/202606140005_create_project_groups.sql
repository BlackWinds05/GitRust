CREATE TABLE project_groups (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(128) UNIQUE NOT NULL,
    display_name VARCHAR(256) NOT NULL,
    description TEXT,
    avatar_url VARCHAR(512),
    created_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

CREATE TABLE group_members (
    group_id UUID NOT NULL REFERENCES project_groups(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role VARCHAR(16) NOT NULL CHECK (role IN ('owner', 'admin', 'member')),
    joined_at TIMESTAMPTZ DEFAULT now(),
    PRIMARY KEY (group_id, user_id)
);
