-- Migration 003: Cost Tracking System
-- Adds tables for tracking AI provider costs and usage

-- Provider usage tracking
CREATE TABLE IF NOT EXISTS provider_usage (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    provider_name TEXT NOT NULL,
    model TEXT NOT NULL,
    timestamp INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    tokens_used INTEGER NOT NULL,
    cost REAL NOT NULL,
    request_type TEXT CHECK(request_type IN ('completion', 'embedding', 'other')),
    success INTEGER NOT NULL DEFAULT 1,
    error_message TEXT,
    user_input TEXT,
    response_length INTEGER
);

CREATE INDEX IF NOT EXISTS idx_usage_timestamp ON provider_usage(timestamp);
CREATE INDEX IF NOT EXISTS idx_usage_provider ON provider_usage(provider_name);
CREATE INDEX IF NOT EXISTS idx_usage_month ON provider_usage(strftime('%Y-%m', timestamp, 'unixepoch'));

-- Cost budgets and limits
CREATE TABLE IF NOT EXISTS cost_budgets (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    budget_type TEXT NOT NULL CHECK(budget_type IN ('monthly', 'daily', 'total')),
    limit_amount REAL NOT NULL,
    provider_name TEXT,  -- NULL means all providers
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    updated_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_budget_type_provider
    ON cost_budgets(budget_type, COALESCE(provider_name, ''));

-- Cost alerts
CREATE TABLE IF NOT EXISTS cost_alerts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    alert_type TEXT NOT NULL CHECK(alert_type IN ('budget_warning', 'budget_exceeded', 'rate_limit')),
    provider_name TEXT,
    message TEXT NOT NULL,
    threshold_percent REAL,
    triggered_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    acknowledged INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_alerts_triggered ON cost_alerts(triggered_at);
CREATE INDEX IF NOT EXISTS idx_alerts_acknowledged ON cost_alerts(acknowledged);

-- Provider statistics (materialized view-like table for performance)
CREATE TABLE IF NOT EXISTS provider_stats (
    provider_name TEXT PRIMARY KEY,
    total_requests INTEGER NOT NULL DEFAULT 0,
    successful_requests INTEGER NOT NULL DEFAULT 0,
    failed_requests INTEGER NOT NULL DEFAULT 0,
    total_tokens INTEGER NOT NULL DEFAULT 0,
    total_cost REAL NOT NULL DEFAULT 0.0,
    last_updated INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
);

-- Insert default budget (monthly $10 limit)
INSERT OR IGNORE INTO cost_budgets (budget_type, limit_amount)
VALUES ('monthly', 10.0);
