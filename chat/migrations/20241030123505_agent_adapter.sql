-- Add migration script here
CREATE TYPE adapter_type AS ENUM ('ollama');

ALTER TABLE
    chat_agents
ADD
    COLUMN adapter adapter_type NOT NULL DEFAULT 'ollama';

ALTER TABLE
    chat_agents
ADD
    COLUMN model VARCHAR NOT NULL DEFAULT 'llama3.2';
