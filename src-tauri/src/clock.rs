use std::sync::atomic::{AtomicI64, Ordering};

/// 当前时刻（Unix 毫秒）。计时和记录都从这里取时间，测试和开发桥可以换成可控的时钟。
pub trait Clock: Send + Sync {
    fn now_ms(&self) -> i64;
}

pub struct SystemClock;

impl Clock for SystemClock {
    fn now_ms(&self) -> i64 {
        chrono::Utc::now().timestamp_millis()
    }
}

/// 系统时间加一个可调的偏移，开发桥用它“快进”时间。
#[derive(Default)]
pub struct OffsetClock {
    offset_ms: AtomicI64,
}

impl OffsetClock {
    pub fn advance(&self, ms: i64) {
        self.offset_ms.fetch_add(ms, Ordering::SeqCst);
    }
}

impl Clock for OffsetClock {
    fn now_ms(&self) -> i64 {
        SystemClock.now_ms() + self.offset_ms.load(Ordering::SeqCst)
    }
}

/// 完全手动的时钟，测试用。
pub struct ManualClock {
    now_ms: AtomicI64,
}

impl ManualClock {
    pub fn new(now_ms: i64) -> Self {
        Self { now_ms: AtomicI64::new(now_ms) }
    }

    pub fn set(&self, ms: i64) {
        self.now_ms.store(ms, Ordering::SeqCst);
    }

    pub fn advance(&self, ms: i64) {
        self.now_ms.fetch_add(ms, Ordering::SeqCst);
    }
}

impl Clock for ManualClock {
    fn now_ms(&self) -> i64 {
        self.now_ms.load(Ordering::SeqCst)
    }
}
