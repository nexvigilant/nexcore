-- brain.db → Postgres translated schema (v6, derived from SQLite tables)
CREATE TABLE IF NOT EXISTS schema_version (version INT PRIMARY KEY, applied_at TIMESTAMPTZ DEFAULT NOW());
CREATE TABLE IF NOT EXISTS sessions (
  id TEXT PRIMARY KEY,
  description TEXT,
  created_at TIMESTAMPTZ DEFAULT NOW(),
  updated_at TIMESTAMPTZ DEFAULT NOW(),
  machine_id TEXT,
  status TEXT DEFAULT 'active'
);
CREATE TABLE IF NOT EXISTS artifacts (
  id BIGSERIAL PRIMARY KEY,
  session_id TEXT REFERENCES sessions(id) ON DELETE CASCADE,
  name TEXT NOT NULL,
  artifact_type TEXT NOT NULL,
  content TEXT,
  metadata JSONB,
  created_at TIMESTAMPTZ DEFAULT NOW(),
  updated_at TIMESTAMPTZ DEFAULT NOW(),
  UNIQUE(session_id, name)
);
CREATE TABLE IF NOT EXISTS artifact_versions (
  id BIGSERIAL PRIMARY KEY,
  artifact_id BIGINT REFERENCES artifacts(id) ON DELETE CASCADE,
  version INT NOT NULL,
  content TEXT,
  created_at TIMESTAMPTZ DEFAULT NOW()
);
CREATE TABLE IF NOT EXISTS autopsy_records (
  session_id TEXT PRIMARY KEY REFERENCES sessions(id) ON DELETE CASCADE,
  outcome_verdict TEXT,
  lessons_count INT DEFAULT 0,
  patterns_count INT DEFAULT 0,
  proposition TEXT,
  rho_status TEXT,
  manifest_files JSONB,
  rc_pdp_proposition INT DEFAULT 0,
  rc_pdp_so_what INT DEFAULT 0,
  rc_pdp_why INT DEFAULT 0,
  rc_hook_gap INT DEFAULT 0,
  chain_level INT DEFAULT 0,
  files_modified INT DEFAULT 0,
  measured BOOLEAN DEFAULT FALSE,
  created_at TIMESTAMPTZ DEFAULT NOW()
);
CREATE TABLE IF NOT EXISTS corrections (
  id BIGSERIAL PRIMARY KEY,
  pattern TEXT NOT NULL,
  correction TEXT NOT NULL,
  context TEXT,
  falsifiable_by TEXT,
  trust_weight REAL DEFAULT 1.0,
  created_at TIMESTAMPTZ DEFAULT NOW(),
  last_demonstrated TIMESTAMPTZ DEFAULT NOW()
);
CREATE TABLE IF NOT EXISTS beliefs (
  id BIGSERIAL PRIMARY KEY,
  statement TEXT NOT NULL,
  evidence TEXT,
  confidence REAL DEFAULT 0.5,
  source_session_id TEXT REFERENCES sessions(id) ON DELETE SET NULL,
  created_at TIMESTAMPTZ DEFAULT NOW()
);
CREATE TABLE IF NOT EXISTS patterns (
  id BIGSERIAL PRIMARY KEY,
  name TEXT UNIQUE NOT NULL,
  description TEXT,
  relevance REAL DEFAULT 0.5,
  grounding JSONB,
  created_at TIMESTAMPTZ DEFAULT NOW()
);
CREATE TABLE IF NOT EXISTS trust_accumulators (
  key TEXT PRIMARY KEY,
  value REAL DEFAULT 0.0,
  updated_at TIMESTAMPTZ DEFAULT NOW()
);
CREATE TABLE IF NOT EXISTS tracked_files (
  path TEXT PRIMARY KEY,
  session_id TEXT REFERENCES sessions(id) ON DELETE CASCADE,
  last_modified TIMESTAMPTZ DEFAULT NOW(),
  checksum TEXT
);
CREATE TABLE IF NOT EXISTS decision_audit (
  id BIGSERIAL PRIMARY KEY,
  session_id TEXT REFERENCES sessions(id) ON DELETE CASCADE,
  decision TEXT NOT NULL,
  rationale TEXT,
  outcome TEXT,
  created_at TIMESTAMPTZ DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_artifacts_session ON artifacts(session_id);
CREATE INDEX IF NOT EXISTS idx_artifacts_type ON artifacts(artifact_type);
CREATE INDEX IF NOT EXISTS idx_autopsy_verdict ON autopsy_records(outcome_verdict);
CREATE INDEX IF NOT EXISTS idx_sessions_created ON sessions(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_patterns_relevance ON patterns(relevance DESC);

-- Vault metadata (full markdown content stays on git repo + GCS; this indexes tags/paths)
CREATE TABLE IF NOT EXISTS vault_notes (
  path TEXT PRIMARY KEY,
  title TEXT,
  folder TEXT,
  tags TEXT[],
  word_count INT,
  updated_at TIMESTAMPTZ DEFAULT NOW(),
  tsv TSVECTOR
);
CREATE INDEX IF NOT EXISTS idx_vault_tsv ON vault_notes USING GIN(tsv);
CREATE INDEX IF NOT EXISTS idx_vault_tags ON vault_notes USING GIN(tags);
CREATE INDEX IF NOT EXISTS idx_vault_folder ON vault_notes(folder);

INSERT INTO schema_version (version) VALUES (6) ON CONFLICT DO NOTHING;
