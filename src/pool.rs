use prometheus::{Encoder, TextEncoder};
use std::sync::Arc;
use tokio::signal;
use tokio::sync::Semaphore;
mod metric_counter;
use metric_counter::BizCounter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 并发限制：最多同时跑4个任务
    let sem = Arc::new(Semaphore::new(4));
    let biz_counter = Arc::new(BizCounter::new());

    // 指标暴露端口，/metrics
    let counter_clone = biz_counter.clone();
    let metric_server = tokio::spawn(async move {
        let addr = ([127, 0, 0, 1], 9090).into();
        axum::Server::bind(&addr)
            .serve(
                axum::Router::new()
                    .route(
                        "/metrics",
                        axum::routing::get(move || {
                            let metric_families = prometheus::gather();
                            async move {
                                let encoder = TextEncoder::new();
                                let mut buffer = Vec::new();
                                encoder.encode(&metric_families, &mut buffer).unwrap();
                                String::from_utf8(buffer).unwrap()
                            }
                        }),
                    )
                    .into_make_service(),
            )
            .await
            .unwrap();
    });

    // 模拟提交10个任务
    for i in 0..10 {
        let sem = sem.clone();
        let biz_counter = biz_counter.clone();
        let task_name = format!("task_{}", i % 3);

        tokio::spawn(async move {
            // 获取信号量许可，控制并发
            let _permit = sem.acquire().await.unwrap();
            println!("run task:{}", task_name);

            // 模拟业务
            if i % 5 != 0 {
                biz_counter.on_task_success(&task_name).await;
            } else {
                biz_counter.on_task_fail(&task_name).await;
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
        });
    }

    // ===== 优雅关闭信号处理 =====
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install ctrl-c handler");
    };

    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    tokio::select! {
        _ = ctrl_c => println!("receive SIGINT, shutting down..."),
        _ = terminate => println!("receive SIGTERM, shutting down..."),
    }

    // 等待后台任务完成，设置超时兜底
    println!("Wait pending tasks, max wait 3s");
    let _ = tokio::time::timeout(tokio::time::Duration::from_secs(3), async {
        // 这里可以加等待任务完成逻辑
    })
    .await;

    println!("shutdown complete");
    Ok(())
}
