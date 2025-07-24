-- 004_add_customer_group.sql
-- 添加客户分组字段

-- 为customers表添加customer_group字段
ALTER TABLE customers 
ADD COLUMN customer_group VARCHAR(20) NOT NULL DEFAULT '团课' 
CHECK (customer_group IN ('团课', '小班', '私教', '教培'));

-- 为现有数据设置默认分组
UPDATE customers SET customer_group = '团课' WHERE customer_group IS NULL;

-- 创建索引以提高按分组查询的性能
CREATE INDEX idx_customers_group ON customers(customer_group);
CREATE INDEX idx_customers_user_group ON customers(user_id, customer_group);