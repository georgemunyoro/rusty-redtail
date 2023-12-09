pub struct SearchThreadPool {
    threads: Vec<SearchThread>,
}

pub struct SearchThread {
    pub id: usize,
    pub nodes: u64,
    pub start_time: u128,
}
