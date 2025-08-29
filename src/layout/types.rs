use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalLayout {
    pub name: String,
    pub color_index: usize,
    pub working_dir: String,
    pub split_type: Option<SplitType>,
    pub children: Vec<TerminalLayout>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum SplitType {
    Horizontal,
    Vertical,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SavedLayout {
    pub version: String,
    pub root: TerminalLayout,
}
