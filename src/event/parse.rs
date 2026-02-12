//! Parsing trait for Event
//!
//! / Event 的解析 trait

use crate::error::Result;

/// Trait for parsing request body data
///
/// / 解析请求体数据的 trait
pub trait EventParse {
    /// Parse JSON body from bytes
    ///
    /// / 从字节解析 JSON 请求体
    fn parse_json<T: serde::de::DeserializeOwned>(&self, bytes: &[u8]) -> Result<T>;

    /// Parse form data from bytes
    ///
    /// / 从字节解析表单数据
    fn parse_form<T: serde::de::DeserializeOwned>(&self, bytes: &[u8]) -> Result<T>;

    /// Parse text body from bytes
    ///
    /// / 从字节解析文本请求体
    fn parse_text(&self, bytes: &[u8]) -> Result<String>;
}
