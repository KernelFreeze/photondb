use std::fmt::Display;

use crate::util::atomic::Counter;

/// Statistics of page store.
#[derive(Clone, Copy, Default)]
pub struct StoreStats {
    /// Statistics of page cache.
    pub page_cache: CacheStats,
    /// Statistics of file reader cache.
    pub file_reader_cache: CacheStats,
    /// Statistics of writebuf.
    pub writebuf: WritebufStats,
    /// Statistics of jobs.
    pub jobs: JobStats,
}

impl StoreStats {
    /// Sub other stats to produce an new stats.
    pub fn sub(&self, o: &StoreStats) -> StoreStats {
        StoreStats {
            page_cache: self.page_cache.sub(&o.page_cache),
            file_reader_cache: self.file_reader_cache.sub(&o.file_reader_cache),
            writebuf: self.writebuf.sub(&o.writebuf),
            jobs: self.jobs.sub(&o.jobs),
        }
    }
}

impl Display for StoreStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "WritebufStats: read_in_buf: {}, read_in_files: {}, read_hit_rate: {}%",
            self.writebuf.read_in_buf,
            self.writebuf.read_in_file,
            (self.writebuf.read_in_buf as f64) * 100.
                / (self.writebuf.read_in_buf + self.writebuf.read_in_file) as f64,
        )?;
        writeln!(
            f,
            "FileReaderCacheStats: lookup_hit: {}, lookup_miss: {}, hit_rate: {}%, insert: {}, active_evict: {}, passive_evict: {}",
            self.file_reader_cache.lookup_hit,
            self.file_reader_cache.lookup_miss,
            (self.file_reader_cache.lookup_hit as f64) * 100.
                / (self.file_reader_cache.lookup_hit + self.file_reader_cache.lookup_miss) as f64,
            self.file_reader_cache.insert,
            self.file_reader_cache.active_evict,
            self.file_reader_cache.passive_evict,
        )?;
        writeln!(
            f,
            "PageCacheStats: lookup_hit: {}, lookup_miss: {}, hit_rate: {}%, insert: {}, active_evict: {}, passive_evict: {}",
            self.page_cache.lookup_hit,
            self.page_cache.lookup_miss,
            (self.page_cache.lookup_hit as f64) * 100.
                / (self.page_cache.lookup_hit + self.page_cache.lookup_miss) as f64,
            self.page_cache.insert,
            self.page_cache.active_evict,
            self.page_cache.passive_evict,
        )?;

        let write_amp = if self.jobs.flush_write_bytes == 0 {
            0.0
        } else {
            let write_bytes = self.jobs.rewrite_bytes + self.jobs.compact_write_bytes;
            (write_bytes as f64) / (self.jobs.flush_write_bytes as f64)
        };
        writeln!(
            f,
            "JobStats: flush_write_bytes: {}, rewrite_bytes: {}, compact_write_bytes: {}, write_amp: {:.2}",
            self.jobs.flush_write_bytes, self.jobs.rewrite_bytes, self.jobs.compact_write_bytes, write_amp
        )
    }
}

/// Statistics of cache.
#[derive(Default, Clone, Debug, Copy)]
pub struct CacheStats {
    pub lookup_hit: u64,
    pub lookup_miss: u64,
    pub insert: u64,
    pub active_evict: u64,
    pub passive_evict: u64,
}

impl CacheStats {
    fn sub(&self, o: &CacheStats) -> CacheStats {
        CacheStats {
            lookup_hit: self.lookup_hit.wrapping_sub(o.lookup_hit),
            lookup_miss: self.lookup_miss.wrapping_sub(o.lookup_miss),
            insert: self.insert.wrapping_sub(o.insert),
            active_evict: self.active_evict.wrapping_sub(o.active_evict),
            passive_evict: self.passive_evict.wrapping_sub(o.passive_evict),
        }
    }

    pub(crate) fn add(&self, o: &CacheStats) -> CacheStats {
        CacheStats {
            lookup_hit: self.lookup_hit.wrapping_add(o.lookup_hit),
            lookup_miss: self.lookup_miss.wrapping_add(o.lookup_miss),
            insert: self.insert.wrapping_add(o.insert),
            active_evict: self.active_evict.wrapping_add(o.active_evict),
            passive_evict: self.passive_evict.wrapping_add(o.passive_evict),
        }
    }
}

#[derive(Default, Clone, Debug, Copy)]
pub struct WritebufStats {
    pub read_in_buf: u64,
    pub read_in_file: u64,
}

impl WritebufStats {
    fn sub(&self, o: &WritebufStats) -> WritebufStats {
        WritebufStats {
            read_in_buf: self.read_in_buf.wrapping_sub(o.read_in_buf),
            read_in_file: self.read_in_file.wrapping_sub(o.read_in_file),
        }
    }
}

#[derive(Default)]
pub(crate) struct AtomicWritebufStats {
    pub(super) read_in_buf: Counter,
    pub(super) read_in_file: Counter,
}

impl AtomicWritebufStats {
    pub(super) fn snapshot(&self) -> WritebufStats {
        WritebufStats {
            read_in_buf: self.read_in_buf.get(),
            read_in_file: self.read_in_file.get(),
        }
    }
}

#[derive(Debug, Copy, Clone, Default)]
pub struct JobStats {
    /// The total write bytes during flush.
    pub flush_write_bytes: u64,
    /// The total rewrite bytes.
    pub rewrite_bytes: u64,
    /// The total bytes write during compaction.
    pub compact_write_bytes: u64,
}

#[derive(Default, Debug)]
pub(crate) struct AtomicJobStats {
    pub(super) flush_write_bytes: Counter,
    pub(super) rewrite_bytes: Counter,
    pub(super) compact_write_bytes: Counter,
}

impl JobStats {
    pub fn sub(&self, o: &Self) -> Self {
        JobStats {
            flush_write_bytes: self.flush_write_bytes.wrapping_sub(o.flush_write_bytes),
            rewrite_bytes: self.rewrite_bytes.wrapping_sub(o.rewrite_bytes),
            compact_write_bytes: self.compact_write_bytes.wrapping_add(o.compact_write_bytes),
        }
    }
}

impl AtomicJobStats {
    pub(crate) fn snapshot(&self) -> JobStats {
        JobStats {
            flush_write_bytes: self.flush_write_bytes.get(),
            rewrite_bytes: self.rewrite_bytes.get(),
            compact_write_bytes: self.compact_write_bytes.get(),
        }
    }
}
