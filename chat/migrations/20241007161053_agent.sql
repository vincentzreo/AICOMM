-- Add migration script here
-- modify chats table to add agents column
ALTER TABLE
    chats
ADD
    COLUMN agents BIGINT [] NOT NULL DEFAULT '{}';

-- modify messages table to add modified_content column
ALTER TABLE
    messages
ADD
    COLUMN modified_content TEXT;

-- create agent_type type
CREATE TYPE agent_type AS ENUM ('proxy', 'reply', 'tap');

-- add chat_agents table
CREATE TABLE chat_agents (
    id BIGSERIAL PRIMARY KEY,
    chat_id BIGINT NOT NULL REFERENCES chats(id),
    name TEXT NOT NULL UNIQUE,
    type agent_type NOT NULL DEFAULT 'reply',
    prompt TEXT NOT NULL,
    args JSONB NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);
