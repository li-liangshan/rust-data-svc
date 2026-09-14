use crate::Result;

// 示例业务服务层，业务逻辑写在这里，handler保持轻薄
#[allow(dead_code)]
pub async fn hello(name: &str) -> Result<String> {
    Ok(format!("Hello, {} !", name))
}
