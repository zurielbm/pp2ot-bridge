/// Timed entry for formatter - represents an item with timing info
#[derive(Debug, Clone, PartialEq)]
pub struct TimedEntry {
    pub item_id: String,
    pub name: String,
    pub item_type: String,
    pub duration: String,
    pub end_time: String,
    pub count_to_end: bool,
    pub link_start: bool,
    /// Optional insertion index for UI placement (0 = start, n = after n-th existing entry)
    pub insertion_index: Option<usize>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct FormatterGroup {
    pub id: String,
    pub name: String,
    pub color: String,
    pub entries: Vec<TimedEntry>,
    pub collapsed: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InsertionMode {
    After,
    Into, // For groups
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum TimeField {
    Duration,
    EndTime,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimeEditContext {
    pub item_idx: usize,
    pub sub_item_idx: Option<usize>,
    pub field: TimeField,
    pub current_value: String,
}

/// Unified item type for the formatter - can be standalone or a group
#[derive(Debug, Clone, PartialEq)]
pub enum FormatterItem {
    Standalone(TimedEntry),
    Group {
        id: String,
        name: String,
        color: String,
        entries: Vec<TimedEntry>,
        collapsed: bool,
    },
    Reference {
        id: String,
        title: String,
        item_type: String,
        mode: InsertionMode,
        time_end: u64, // End time of the referenced event (ms)
    },
}
