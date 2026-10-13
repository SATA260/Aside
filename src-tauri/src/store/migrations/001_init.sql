-- 时间一律存 Unix 毫秒（UTC），按天统计时再换算成本地时间。

CREATE TABLE subjects (
    id          INTEGER PRIMARY KEY,
    name        TEXT    NOT NULL UNIQUE,
    color       TEXT    NOT NULL,
    sort        INTEGER NOT NULL,
    archived_at INTEGER,
    is_fallback INTEGER NOT NULL DEFAULT 0
);

INSERT INTO subjects (name, color, sort, is_fallback) VALUES
    ('政治',   '#C98B7B', 0, 0),
    ('英语',   '#7FA3C7', 1, 0),
    ('数学',   '#8DB59A', 2, 0),
    ('专业课', '#C7AE7F', 3, 0),
    ('其他',   '#A3A9B4', 4, 1);

-- 一段专注。mode: countdown | countup | mock；source: timer | manual
CREATE TABLE sessions (
    id          INTEGER PRIMARY KEY,
    subject_id  INTEGER NOT NULL REFERENCES subjects (id),
    mode        TEXT    NOT NULL,
    planned_ms  INTEGER,
    started_at  INTEGER NOT NULL,
    ended_at    INTEGER NOT NULL,
    focused_ms  INTEGER NOT NULL,
    task        TEXT    NOT NULL DEFAULT '',
    note        TEXT    NOT NULL DEFAULT '',
    source      TEXT    NOT NULL
);
CREATE INDEX sessions_started_at ON sessions (started_at);

-- 一段专注里真正在跑的时间段，暂停会把一段专注切成几段
CREATE TABLE segments (
    session_id INTEGER NOT NULL REFERENCES sessions (id) ON DELETE CASCADE,
    start_at   INTEGER NOT NULL,
    end_at     INTEGER NOT NULL
);
CREATE INDEX segments_session ON segments (session_id);
CREATE INDEX segments_start ON segments (start_at);

CREATE TABLE mock_scores (
    session_id INTEGER PRIMARY KEY REFERENCES sessions (id) ON DELETE CASCADE,
    score      INTEGER NOT NULL,
    full_score INTEGER NOT NULL
);

-- 导入的背景音，文件放在数据文件夹的 sounds/ 下
CREATE TABLE sounds (
    id          INTEGER PRIMARY KEY,
    name        TEXT    NOT NULL,
    file        TEXT    NOT NULL,
    gain_db     REAL    NOT NULL DEFAULT 0,
    duration_ms INTEGER NOT NULL DEFAULT 0,
    created_at  INTEGER NOT NULL
);

-- 导入的背景图片，原图和压暗版放在数据文件夹的 scenes/ 下
CREATE TABLE images (
    id         INTEGER PRIMARY KEY,
    name       TEXT    NOT NULL,
    file       TEXT    NOT NULL,
    created_at INTEGER NOT NULL
);

CREATE TABLE settings (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

-- 正在进行的专注，用来在异常退出后恢复；最多一行
CREATE TABLE active_timer (
    id           INTEGER PRIMARY KEY CHECK (id = 1),
    state        TEXT    NOT NULL,
    heartbeat_at INTEGER NOT NULL
);
