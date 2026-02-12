//! Application state extraction
//!
//! / 应用状态提取

use crate::{
    Event,
    error::{Result, RouteError},
};

/// Get application state by type
///
/// / 根据类型获取应用状态
///
/// # Type Parameters
///
/// # 类型参数
///
/// - `T` - The type to retrieve (must be `Clone + Send + Sync + 'static`)
///   要检索的类型（必须是 `Clone + Send + Sync + 'static`）
///
/// # Errors
///
/// # 错误
///
/// Returns `RouteError::Internal` if the state is not found.
///
/// 如果未找到状态，返回 `RouteError::Internal`。
///
/// # Example
///
/// # 示例
///
/// ```rust,ignore
/// struct DatabasePool {
///     // ...
/// }
///
/// let pool = get_state::<DatabasePool>(&event)?;
/// ```
pub fn get_state<T: Clone + Send + Sync + 'static>(event: &Event) -> Result<T> {
    event
        .state()
        .ok_or_else(|| RouteError::Internal(anyhow::anyhow!("State not found")))
}
