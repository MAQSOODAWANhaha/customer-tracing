-- 创建客户表
CREATE TABLE customers (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name VARCHAR(100) NOT NULL,
    phone VARCHAR(20),
    address TEXT,
    notes TEXT,
    rate INTEGER DEFAULT 0 CHECK(rate >= 0 AND rate <= 5),
    user_id INTEGER NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    is_deleted BOOLEAN DEFAULT FALSE,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

-- 创建索引
CREATE INDEX idx_customers_user_id ON customers(user_id);
CREATE INDEX idx_customers_is_deleted ON customers(is_deleted);
CREATE INDEX idx_customers_name ON customers(name);