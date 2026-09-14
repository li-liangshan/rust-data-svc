use prometheus::{register_int_counter, register_int_gauge, IntCounter, IntGauge};
use std::collections::HashMap;
use tokio::sync::RwLock;

/// 业务内存计数器 + Prom指标封装
#[derive(Debug, Clone)]
pub struct BizCounter {
    // 内存hashmap，多任务读写加RwLock
    inner: RwLock<HashMap<String, u64>>,

    // Prometheus指标
    pub task_total: IntCounter,
    pub task_success: IntCounter,
    pub task_fail: IntCounter,
    pub map_key_count: IntGauge, // HashMap当前key数量监控
}

impl BizCounter {
    pub fn new() -> Self {
        let task_total =
            register_int_counter!("biz_task_total", "total received task count").unwrap();

        let task_success = register_int_counter!("biz_task_success", "success task count").unwrap();

        let task_fail = register_int_counter!("biz_task_fail", "failed task count").unwrap();

        let map_key_count =
            register_int_gauge!("biz_hashmap_key_num", "current key count inside hashmap").unwrap();

        Self {
            inner: RwLock::new(HashMap::new()),
            task_total,
            task_success,
            task_fail,
            map_key_count,
        }
    }

    /// 任务成功，更新HashMap + prom指标
    pub async fn on_task_success(&self, task_name: &str) {
        self.task_total.inc();
        self.task_success.inc();

        let mut map = self.inner.write().await;
        // entry API，一次hash查找，不存在初始化为0
        let val = map.entry(task_name.to_string()).or_insert(0);
        *val += 1;

        // 更新 gauge：当前hashmap的key总数
        self.map_key_count.set(map.len() as i64);
    }

    /// 任务失败
    pub async fn on_task_fail(&self, task_name: &str) {
        self.task_total.inc();
        self.task_fail.inc();

        let mut map = self.inner.write().await;
        let val = map.entry(task_name.to_string()).or_insert(0);
        *val += 1;
        self.map_key_count.set(map.len() as i64);
    }

    /// 获取某个任务的计数
    pub async fn get_count(&self, task_name: &str) -> Option<u64> {
        let map = self.inner.read().await;
        map.get(&task_name.to_string()).copied()
    }

    /// 清空内存hashmap
    pub async fn reset(&self) {
        let mut map = self.inner.write().await;
        map.clear();
        self.map_key_count.set(0);
    }
}
