//! Handles storage of data kept between launches.

use once_cell::sync::OnceCell;
use std::collections::HashMap;

static STORAGE: OnceCell<HashMap<String, serde_json::Value>> = OnceCell::new();

pub fn init() {}
