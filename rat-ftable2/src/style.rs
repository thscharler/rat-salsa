use crate::_private::NonExhaustive;
use rat_scrolled::ScrollStyle;
use ratatui_core::style::Style;
use ratatui_widgets::block::Block;

/// Combined style.
#[derive(Debug, Clone)]
pub struct TableStyle {
    pub style: Style,
    pub block: Option<Block<'static>>,
    pub border_style: Option<Style>,
    pub title_style: Option<Style>,
    pub scroll: Option<ScrollStyle>,
    pub header: Option<Style>,
    pub footer: Option<Style>,
    pub focus_style: Option<Style>,

    pub select_row: Option<Style>,
    pub select_column: Option<Style>,
    pub select_cell: Option<Style>,
    pub select_header: Option<Style>,
    pub select_footer: Option<Style>,

    pub show_row_focus: bool,
    pub show_column_focus: bool,
    pub show_cell_focus: bool,
    pub show_header_focus: bool,
    pub show_footer_focus: bool,

    pub non_exhaustive: NonExhaustive,
}

impl Default for TableStyle {
    fn default() -> Self {
        Self {
            style: Default::default(),
            header: Default::default(),
            footer: Default::default(),
            select_row: Default::default(),
            select_column: Default::default(),
            select_cell: Default::default(),
            select_header: Default::default(),
            select_footer: Default::default(),
            show_row_focus: true,
            show_column_focus: Default::default(),
            show_cell_focus: Default::default(),
            show_header_focus: Default::default(),
            show_footer_focus: Default::default(),
            focus_style: Default::default(),
            block: Default::default(),
            border_style: Default::default(),
            title_style: Default::default(),
            scroll: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}
