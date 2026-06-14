CREATE TABLE wiki_pages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    repository_id UUID NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    title VARCHAR(256) NOT NULL,
    slug VARCHAR(256) NOT NULL,
    content TEXT NOT NULL,
    author_id UUID NOT NULL REFERENCES users(id),
    revision INT DEFAULT 1,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE (repository_id, slug, revision)
);

CREATE INDEX idx_wiki_repo_slug ON wiki_pages(repository_id, slug);
