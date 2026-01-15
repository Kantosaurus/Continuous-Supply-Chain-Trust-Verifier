-- =============================================================================
-- SCTV Database Initialization Script for Docker
-- =============================================================================
-- This script is executed when the PostgreSQL container starts for the first time.
-- It sets up the required extensions that must be created by a superuser.
--
-- Note: The actual schema migrations are run by the API server on startup.
-- This script only handles extensions and initial configuration.
-- =============================================================================

-- Enable required PostgreSQL extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pg_trgm";

-- Create application role (optional, for production hardening)
DO $$
BEGIN
    IF NOT EXISTS (SELECT FROM pg_catalog.pg_roles WHERE rolname = 'sctv_app') THEN
        CREATE ROLE sctv_app WITH LOGIN PASSWORD 'sctv_app_password';
    END IF;
END
$$;

-- Grant permissions to application role
GRANT CONNECT ON DATABASE sctv TO sctv_app;
GRANT USAGE ON SCHEMA public TO sctv_app;

-- The schema will be created by migrations, but grant default privileges
ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO sctv_app;
ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT USAGE, SELECT ON SEQUENCES TO sctv_app;

-- Log successful initialization
DO $$
BEGIN
    RAISE NOTICE 'SCTV database initialized successfully';
END
$$;
