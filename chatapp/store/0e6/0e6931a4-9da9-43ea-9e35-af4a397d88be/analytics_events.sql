ATTACH TABLE _ UUID 'c9d40c34-309e-4cd0-b2ad-1771eade374a'
(
    `client_id` String,
    `session_id` String,
    `duration` UInt32,
    `app_version` String,
    `system_os` String,
    `system_arch` String,
    `system_locale` String,
    `system_timezone` String,
    `user_id` Nullable(String),
    `ip` Nullable(String),
    `user_agent` Nullable(String),
    `geo_country` Nullable(String),
    `geo_region` Nullable(String),
    `geo_city` Nullable(String),
    `client_ts` DateTime64(3),
    `server_ts` DateTime64(3),
    `event_type` String,
    `exit_code` Nullable(String),
    `login_email` Nullable(String),
    `logout_email` Nullable(String),
    `register_email` Nullable(String),
    `register_workspace_id` Nullable(String),
    `chat_created_workspace_id` Nullable(String),
    `message_chat_id` Nullable(String),
    `message_type` Nullable(String),
    `message_size` Nullable(Int32),
    `message_total_files` Nullable(Int32),
    `chat_joined_id` Nullable(String),
    `chat_left_id` Nullable(String),
    `navigation_from` Nullable(String),
    `navigation_to` Nullable(String)
)
ENGINE = MergeTree
ORDER BY (event_type, session_id, client_id, server_ts)
SETTINGS index_granularity = 8192
