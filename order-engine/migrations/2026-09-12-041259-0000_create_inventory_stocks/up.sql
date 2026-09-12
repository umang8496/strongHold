-- Inventory tracking per product and warehouse location
CREATE TABLE IF NOT EXISTS inventory.stocks (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    product_id UUID NOT NULL UNIQUE,
    warehouse_code VARCHAR(32) NOT NULL DEFAULT 'WH-DEFAULT',
    available_quantity INT NOT NULL DEFAULT 0 CHECK (available_quantity >= 0),
    reserved_quantity INT NOT NULL DEFAULT 0 CHECK (reserved_quantity >= 0),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index product_id for O(1) stock lookups during order reservation
CREATE INDEX idx_inventory_stocks_product_id ON inventory.stocks(product_id);
