-- 创建客户追踪记录表
CREATE TABLE customer_tracks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    customer_id INTEGER NOT NULL,
    content TEXT NOT NULL,
    next_action VARCHAR(64) DEFAULT '继续跟进' CHECK(next_action IN ('继续跟进', '结束跟进')),
    track_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    next_track_time TIMESTAMP,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (customer_id) REFERENCES customers(id) ON DELETE CASCADE
);

-- 创建索引
CREATE INDEX idx_customer_tracks_customer_id ON customer_tracks(customer_id);
CREATE INDEX idx_customer_tracks_track_time ON customer_tracks(track_time);