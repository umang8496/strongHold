-- Orders envelope table
CREATE TABLE IF NOT EXISTS orders.orders (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    customer_id UUID NOT NULL,
    status VARCHAR(32) NOT NULL DEFAULT 'DRAFT',
    total_amount_cents BIGINT NOT NULL DEFAULT 0 CHECK (total_amount_cents >= 0),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Line items belonging to an order
CREATE TABLE IF NOT EXISTS orders.order_items (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    order_id UUID NOT NULL REFERENCES orders.orders(id) ON DELETE CASCADE,
    product_id UUID NOT NULL,
    quantity INT NOT NULL CHECK (quantity > 0),
    unit_price_cents BIGINT NOT NULL CHECK (unit_price_cents >= 0),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_orders_customer_id ON orders.orders(customer_id);
CREATE INDEX idx_orders_status ON orders.orders(status);
CREATE INDEX idx_order_items_order_id ON orders.order_items(order_id);
