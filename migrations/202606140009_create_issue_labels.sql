CREATE TABLE issue_labels (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    repository_id UUID NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    name VARCHAR(64) NOT NULL,
    color VARCHAR(7) NOT NULL,
    description VARCHAR(256),
    created_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE (repository_id, name)
);

CREATE TABLE issue_label_assignments (
    issue_id UUID NOT NULL REFERENCES issues(id) ON DELETE CASCADE,
    label_id UUID NOT NULL REFERENCES issue_labels(id) ON DELETE CASCADE,
    PRIMARY KEY (issue_id, label_id)
);

CREATE TABLE issue_assignees (
    issue_id UUID NOT NULL REFERENCES issues(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    PRIMARY KEY (issue_id, user_id)
);
