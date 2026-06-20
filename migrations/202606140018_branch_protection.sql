CREATE TABLE branch_protection_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    repository_id UUID NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    branch_pattern VARCHAR(255) NOT NULL,
    require_approvals INT DEFAULT 0,
    block_force_push BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT now()
);
CREATE INDEX idx_branch_prot_repo ON branch_protection_rules(repository_id);
