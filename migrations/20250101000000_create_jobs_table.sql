-- Create jobs table
CREATE TABLE IF NOT EXISTS jobs (
    id BIGSERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('Queued', 'Processing', 'Completed', 'Failed')),
    retry_count INTEGER NOT NULL DEFAULT 0,
    max_retries INTEGER NOT NULL DEFAULT 3,
    created_at BIGINT NOT NULL,
    started_at BIGINT,
    completed_at BIGINT,
    failed_reason TEXT
);

-- Create index on status for faster queue queries
CREATE INDEX idx_jobs_status ON jobs(status);

-- Create index on created_at for FIFO ordering
CREATE INDEX idx_jobs_created_at ON jobs(created_at);
