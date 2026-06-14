CREATE TABLE captcha_challenges (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    token VARCHAR(128) UNIQUE NOT NULL,
    answer VARCHAR(16) NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX idx_captcha_token ON captcha_challenges(token);
CREATE INDEX idx_captcha_expires ON captcha_challenges(expires_at);
