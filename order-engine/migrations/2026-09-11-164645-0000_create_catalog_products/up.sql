-- Ensure UUID generator extension exists
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Create the products table under the 'catalog' schema
CREATE TABLE IF NOT EXISTS catalog.products (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    sku VARCHAR(64) NOT NULL UNIQUE,
    title VARCHAR(255) NOT NULL,
    description TEXT,
    price_cents BIGINT NOT NULL CHECK (price_cents >= 0),
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index frequently queried columns (search and active listing filtering)
CREATE INDEX idx_catalog_products_sku ON catalog.products(sku);
CREATE INDEX idx_catalog_products_active ON catalog.products(is_active);
