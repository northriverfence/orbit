-- Migration 002: Learning System
-- Adds tables for user preferences, command analytics, pattern recognition, and insights

-- User preferences table
CREATE TABLE IF NOT EXISTS preferences (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    key TEXT NOT NULL UNIQUE,
    value TEXT NOT NULL,
    updated_at INTEGER NOT NULL,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_preferences_key ON preferences(key);

-- Command analytics table
-- Tracks every command execution for learning and improvement
CREATE TABLE IF NOT EXISTS command_analytics (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    original_input TEXT NOT NULL,
    suggested_command TEXT,
    executed_command TEXT,
    result TEXT NOT NULL CHECK(result IN ('success', 'failed', 'rejected', 'edited')),
    execution_time_ms INTEGER,
    exit_code INTEGER,
    timestamp INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    context_hash TEXT,
    provider TEXT,
    cwd TEXT,
    shell TEXT
);

CREATE INDEX IF NOT EXISTS idx_analytics_timestamp ON command_analytics(timestamp);
CREATE INDEX IF NOT EXISTS idx_analytics_result ON command_analytics(result);
CREATE INDEX IF NOT EXISTS idx_analytics_context ON command_analytics(context_hash);
CREATE INDEX IF NOT EXISTS idx_analytics_provider ON command_analytics(provider);

-- Command patterns table
-- Stores recognized patterns for quick lookup and suggestion ranking
CREATE TABLE IF NOT EXISTS command_patterns (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    pattern TEXT NOT NULL UNIQUE,
    frequency INTEGER DEFAULT 1,
    success_count INTEGER DEFAULT 0,
    failure_count INTEGER DEFAULT 0,
    success_rate REAL GENERATED ALWAYS AS (
        CASE
            WHEN (success_count + failure_count) = 0 THEN 0.0
            ELSE CAST(success_count AS REAL) / (success_count + failure_count)
        END
    ) VIRTUAL,
    last_used INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    preferred_translation TEXT,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_patterns_frequency ON command_patterns(frequency DESC);
CREATE INDEX IF NOT EXISTS idx_patterns_success_rate ON command_patterns(success_rate DESC);
CREATE INDEX IF NOT EXISTS idx_patterns_last_used ON command_patterns(last_used DESC);

-- Learning insights table
-- Stores AI-generated insights about user behavior
CREATE TABLE IF NOT EXISTS insights (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    category TEXT NOT NULL CHECK(category IN ('usage', 'preference', 'pattern', 'error', 'optimization')),
    insight TEXT NOT NULL,
    confidence REAL CHECK(confidence >= 0.0 AND confidence <= 1.0),
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    relevant_until INTEGER,
    acknowledged INTEGER DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_insights_category ON insights(category);
CREATE INDEX IF NOT EXISTS idx_insights_confidence ON insights(confidence DESC);
CREATE INDEX IF NOT EXISTS idx_insights_created ON insights(created_at DESC);

-- Insert default preferences
INSERT OR IGNORE INTO preferences (key, value, updated_at) VALUES
    ('verbosity', 'normal', strftime('%s', 'now')),
    ('provider', 'default', strftime('%s', 'now')),
    ('learning_mode', 'adaptive', strftime('%s', 'now')),
    ('suggestions_enabled', 'true', strftime('%s', 'now')),
    ('analytics_enabled', 'true', strftime('%s', 'now'));
