CREATE TABLE repository_transfers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    repository_id UUID NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    from_owner_type VARCHAR(8) NOT NULL,
    from_owner_id UUID NOT NULL,
    to_owner_type VARCHAR(8) NOT NULL,
    to_owner_id UUID NOT NULL,
    transferred_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT now()
);
