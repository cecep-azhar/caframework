-- 0001_baseline_schema.sql
CREATE TABLE app_kv (
    k TEXT PRIMARY KEY,
    v TEXT NOT NULL
);

CREATE TABLE profiles (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    role TEXT NOT NULL,
    avatar TEXT,
    pin TEXT,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    deleted_at INTEGER,
    rev INTEGER NOT NULL DEFAULT 1,
    origin_device_id TEXT NOT NULL,
    owner_profile_id TEXT NOT NULL,
    visibility TEXT NOT NULL CHECK (visibility IN ('shared','private_summary','private'))
);

CREATE TABLE notes (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    content TEXT,
    tags TEXT,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    deleted_at INTEGER,
    rev INTEGER NOT NULL DEFAULT 1,
    origin_device_id TEXT NOT NULL,
    owner_profile_id TEXT NOT NULL,
    visibility TEXT NOT NULL CHECK (visibility IN ('shared','private_summary','private'))
);

CREATE TABLE change_log (
    seq INTEGER PRIMARY KEY AUTOINCREMENT,
    entity TEXT NOT NULL,
    entity_id TEXT NOT NULL,
    op TEXT NOT NULL CHECK (op IN ('insert','update','delete')),
    rev INTEGER NOT NULL,
    at INTEGER NOT NULL,
    profile_id TEXT,
    device_id TEXT NOT NULL
);

