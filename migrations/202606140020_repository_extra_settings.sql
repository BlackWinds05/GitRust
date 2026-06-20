ALTER TABLE repositories ADD COLUMN IF NOT EXISTS max_file_size_mb INT DEFAULT 100;
ALTER TABLE repositories ADD COLUMN IF NOT EXISTS enable_notifications BOOLEAN DEFAULT true;
