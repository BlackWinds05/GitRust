CREATE TABLE invite_codes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    code VARCHAR(32) UNIQUE NOT NULL,
    group_id UUID NOT NULL REFERENCES project_groups(id) ON DELETE CASCADE,
    created_by UUID NOT NULL REFERENCES users(id),
    max_uses INT,
    current_uses INT DEFAULT 0,
    expires_at TIMESTAMPTZ,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX idx_invite_codes_code ON invite_codes(code);
