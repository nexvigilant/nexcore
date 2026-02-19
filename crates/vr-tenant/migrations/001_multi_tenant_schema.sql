-- PRPaaS Multi-Tenant Schema with Row-Level Security
-- Migration: 001_multi_tenant_schema
-- Created: 2026-02-17
--
-- Implements tenant isolation at the database level using PostgreSQL RLS.
-- Every data table has tenant_id + RLS policy + scoped unique constraints.
--
-- Connection setup: SET app.current_tenant_id = '<uuid>' before queries.
-- This is set by the Axum middleware after JWT verification.

-- ═══════════════════════════════════════════════════════════════
-- EXTENSIONS
-- ═══════════════════════════════════════════════════════════════

CREATE EXTENSION IF NOT EXISTS "pgcrypto";  -- gen_random_uuid()

-- ═══════════════════════════════════════════════════════════════
-- PLATFORM TABLES (manage the platform itself)
-- ═══════════════════════════════════════════════════════════════

CREATE TABLE tenants (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name            TEXT NOT NULL,
    slug            TEXT NOT NULL UNIQUE,
    tier            TEXT NOT NULL DEFAULT 'academic',
    status          TEXT NOT NULL DEFAULT 'provisioning',
    data_classification TEXT NOT NULL DEFAULT 'internal',
    trial_ends_at   TIMESTAMPTZ,
    settings        JSONB DEFAULT '{}',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_tenants_slug ON tenants(slug);
CREATE INDEX idx_tenants_status ON tenants(status);

-- Trigger to auto-update updated_at
CREATE OR REPLACE FUNCTION update_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = now();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER tenants_updated_at
    BEFORE UPDATE ON tenants
    FOR EACH ROW EXECUTE FUNCTION update_updated_at();

-- ═══════════════════════════════════════════════════════════════
-- TENANT USERS (team members within a tenant)
-- ═══════════════════════════════════════════════════════════════

CREATE TABLE tenant_users (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id       UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    firebase_uid    TEXT NOT NULL,
    email           TEXT NOT NULL,
    display_name    TEXT,
    role            TEXT NOT NULL DEFAULT 'viewer',
    status          TEXT NOT NULL DEFAULT 'active',
    invited_by      UUID REFERENCES tenant_users(id),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(tenant_id, email),
    UNIQUE(tenant_id, firebase_uid)
);

CREATE INDEX idx_tenant_users_tenant ON tenant_users(tenant_id);
CREATE INDEX idx_tenant_users_firebase ON tenant_users(firebase_uid);

ALTER TABLE tenant_users ENABLE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON tenant_users
    USING (tenant_id = current_setting('app.current_tenant_id', true)::UUID);

-- ═══════════════════════════════════════════════════════════════
-- SUBSCRIPTIONS (billing and tier tracking)
-- ═══════════════════════════════════════════════════════════════

CREATE TABLE subscriptions (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id       UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    stripe_subscription_id TEXT,
    stripe_customer_id     TEXT,
    tier            TEXT NOT NULL,
    status          TEXT NOT NULL DEFAULT 'active',
    current_period_start TIMESTAMPTZ,
    current_period_end   TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_subscriptions_tenant ON subscriptions(tenant_id);
CREATE INDEX idx_subscriptions_stripe ON subscriptions(stripe_subscription_id);

ALTER TABLE subscriptions ENABLE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON subscriptions
    USING (tenant_id = current_setting('app.current_tenant_id', true)::UUID);

CREATE TRIGGER subscriptions_updated_at
    BEFORE UPDATE ON subscriptions
    FOR EACH ROW EXECUTE FUNCTION update_updated_at();

-- ═══════════════════════════════════════════════════════════════
-- USAGE EVENTS (metering for billing)
-- ═══════════════════════════════════════════════════════════════

CREATE TABLE usage_events (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id       UUID NOT NULL,
    user_id         UUID,
    event_type      TEXT NOT NULL,
    quantity         DOUBLE PRECISION NOT NULL,
    metadata        JSONB,
    timestamp       TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_usage_events_tenant ON usage_events(tenant_id);
CREATE INDEX idx_usage_events_type ON usage_events(event_type);
CREATE INDEX idx_usage_events_timestamp ON usage_events(timestamp);

ALTER TABLE usage_events ENABLE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON usage_events
    USING (tenant_id = current_setting('app.current_tenant_id', true)::UUID);

-- ═══════════════════════════════════════════════════════════════
-- RESEARCH PROGRAMS (tenant-scoped)
-- ═══════════════════════════════════════════════════════════════

CREATE TABLE programs (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id       UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    code_name       TEXT NOT NULL,
    therapeutic_area TEXT NOT NULL,
    target_name     TEXT NOT NULL,
    target_gene     TEXT,
    current_stage   TEXT NOT NULL DEFAULT 'target_validation',
    status          TEXT NOT NULL DEFAULT 'active',
    started_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    budget_total    NUMERIC(12,2),
    budget_spent    NUMERIC(12,2) DEFAULT 0,
    UNIQUE(tenant_id, code_name)
);

CREATE INDEX idx_programs_tenant ON programs(tenant_id);
CREATE INDEX idx_programs_status ON programs(status);

ALTER TABLE programs ENABLE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON programs
    USING (tenant_id = current_setting('app.current_tenant_id', true)::UUID);

CREATE TRIGGER programs_updated_at
    BEFORE UPDATE ON programs
    FOR EACH ROW EXECUTE FUNCTION update_updated_at();

-- ═══════════════════════════════════════════════════════════════
-- MARKETPLACE TABLES
-- ═══════════════════════════════════════════════════════════════

CREATE TABLE marketplace_providers (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    provider_type   TEXT NOT NULL,
    name            TEXT NOT NULL,
    description     TEXT,
    capabilities    JSONB,
    pricing         JSONB,
    rating          DOUBLE PRECISION DEFAULT 0,
    review_count    INTEGER DEFAULT 0,
    status          TEXT NOT NULL DEFAULT 'pending_review',
    contact_email   TEXT NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE marketplace_services (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    provider_id     UUID NOT NULL REFERENCES marketplace_providers(id),
    service_type    TEXT NOT NULL,
    name            TEXT NOT NULL,
    description     TEXT,
    pricing_model   TEXT NOT NULL,
    price_cents     BIGINT,
    currency        TEXT DEFAULT 'USD',
    turnaround_days INTEGER,
    specifications  JSONB,
    status          TEXT NOT NULL DEFAULT 'active',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE marketplace_orders (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id       UUID NOT NULL REFERENCES tenants(id),
    service_id      UUID NOT NULL REFERENCES marketplace_services(id),
    provider_id     UUID NOT NULL REFERENCES marketplace_providers(id),
    program_id      UUID REFERENCES programs(id),
    status          TEXT NOT NULL DEFAULT 'pending',
    order_details   JSONB NOT NULL,
    quoted_price_cents BIGINT,
    actual_price_cents BIGINT,
    commission_cents   BIGINT,
    submitted_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    completed_at    TIMESTAMPTZ
);

CREATE INDEX idx_marketplace_orders_tenant ON marketplace_orders(tenant_id);

ALTER TABLE marketplace_orders ENABLE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON marketplace_orders
    USING (tenant_id = current_setting('app.current_tenant_id', true)::UUID);

-- ═══════════════════════════════════════════════════════════════
-- ML MODELS (platform + marketplace)
-- ═══════════════════════════════════════════════════════════════

CREATE TABLE ml_models (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    creator_id      UUID REFERENCES marketplace_providers(id),
    name            TEXT NOT NULL,
    model_type      TEXT NOT NULL,
    description     TEXT,
    version         TEXT NOT NULL,
    artifact_path   TEXT NOT NULL,
    benchmark_results JSONB,
    is_platform_model BOOLEAN DEFAULT FALSE,
    pricing_cents   BIGINT,
    usage_count     BIGINT DEFAULT 0,
    rating          DOUBLE PRECISION DEFAULT 0,
    status          TEXT NOT NULL DEFAULT 'active',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- ═══════════════════════════════════════════════════════════════
-- TEAM INVITATIONS
-- ═══════════════════════════════════════════════════════════════

CREATE TABLE team_invitations (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id       UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    email           TEXT NOT NULL,
    role            TEXT NOT NULL DEFAULT 'viewer',
    invited_by      UUID NOT NULL REFERENCES tenant_users(id),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at      TIMESTAMPTZ NOT NULL DEFAULT (now() + interval '7 days'),
    accepted_at     TIMESTAMPTZ,
    status          TEXT NOT NULL DEFAULT 'pending'
);

CREATE INDEX idx_invitations_tenant ON team_invitations(tenant_id);
CREATE INDEX idx_invitations_email ON team_invitations(email);

ALTER TABLE team_invitations ENABLE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON team_invitations
    USING (tenant_id = current_setting('app.current_tenant_id', true)::UUID);

-- ═══════════════════════════════════════════════════════════════
-- HELPER: Set tenant context for RLS
-- ═══════════════════════════════════════════════════════════════

-- Call this at the start of every tenant-scoped database session:
--   SELECT set_tenant_context('tenant-uuid-here');
CREATE OR REPLACE FUNCTION set_tenant_context(p_tenant_id UUID)
RETURNS VOID AS $$
BEGIN
    PERFORM set_config('app.current_tenant_id', p_tenant_id::TEXT, true);
END;
$$ LANGUAGE plpgsql;

-- Verify tenant context is set (for debugging)
CREATE OR REPLACE FUNCTION get_tenant_context()
RETURNS UUID AS $$
BEGIN
    RETURN current_setting('app.current_tenant_id', true)::UUID;
END;
$$ LANGUAGE plpgsql;
