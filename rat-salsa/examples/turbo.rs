//!
//! Copies the menu-structure of Turbo Pascal 7.0.
//!
//! and mimics the style. Turns out the base16 theme doesn't
//! look too bad.
//!
//!

use crate::app::{Scenery, SceneryState};
use crate::config::TurboConfig;
use crate::global::GlobalState;
use crate::message::TurboEvent;
use crate::theme::TurboTheme;
use anyhow::Error;
use rat_salsa::poll::PollCrossterm;
use rat_salsa::{run_tui, RunConfig};
use rat_theme2::schemes::BASE16;
use std::fs;

type AppContext<'a> = rat_salsa::AppContext<'a, GlobalState, TurboEvent, Error>;
type RenderContext<'a> = rat_salsa::RenderContext<'a, GlobalState>;

fn main() -> Result<(), Error> {
    setup_logging()?;

    let config = TurboConfig::default();
    let theme = TurboTheme::new("Base16".into(), BASE16);
    let mut global = GlobalState::new(config, theme);

    let app = Scenery;
    let mut state = SceneryState::default();

    run_tui(
        app,
        &mut global,
        &mut state,
        RunConfig::default()? //
            .poll(PollCrossterm),
    )?;

    Ok(())
}

/// Globally accessible data/state.
pub mod global {
    use crate::config::TurboConfig;
    use crate::theme::TurboTheme;
    use std::rc::Rc;

    #[derive(Debug)]
    pub struct GlobalState {
        pub cfg: TurboConfig,
        pub theme: Rc<TurboTheme>,
    }

    impl GlobalState {
        pub fn new(cfg: TurboConfig, theme: TurboTheme) -> Self {
            Self {
                cfg,
                theme: Rc::new(theme),
            }
        }
    }
}

/// Configuration.
pub mod config {
    #[derive(Debug, Default)]
    pub struct TurboConfig {}
}

/// Application wide messages.
pub mod message {
    use crossterm::event::Event;

    #[derive(Debug)]
    pub enum TurboEvent {
        Event(crossterm::event::Event),
        Message(String),
        Status(usize, String),
    }

    impl From<crossterm::event::Event> for TurboEvent {
        fn from(value: Event) -> Self {
            Self::Event(value)
        }
    }
}

pub mod app {
    use crate::global::GlobalState;
    use crate::message::TurboEvent;
    use crate::turbo::{Turbo, TurboState};
    use crate::{AppContext, RenderContext};
    use anyhow::Error;
    use rat_salsa::{AppState, AppWidget, Control};
    use rat_widget::event::{ct_event, ConsumedEvent, Dialog, HandleEvent, Regular};
    use rat_widget::focus::FocusBuilder;
    use rat_widget::msgdialog::{MsgDialog, MsgDialogState};
    use rat_widget::statusline::{StatusLine, StatusLineState};
    use rat_widget::util::fill_buf_area;
    use ratatui::buffer::Buffer;
    use ratatui::layout::{Constraint, Layout, Rect};
    use ratatui::widgets::StatefulWidget;
    use std::time::{Duration, SystemTime};

    #[derive(Debug)]
    pub struct Scenery;

    #[derive(Debug, Default)]
    pub struct SceneryState {
        pub turbo: TurboState,
        pub status: StatusLineState,
        pub error_dlg: MsgDialogState,
    }

    impl AppWidget<GlobalState, TurboEvent, Error> for Scenery {
        type State = SceneryState;

        fn render(
            &self,
            area: Rect,
            buf: &mut Buffer,
            state: &mut Self::State,
            ctx: &mut RenderContext<'_>,
        ) -> Result<(), Error> {
            let t0 = SystemTime::now();
            let theme = ctx.g.theme.clone();

            let layout = Layout::vertical([
                Constraint::Length(1),
                Constraint::Fill(1),
                Constraint::Length(1),
            ])
            .split(area);

            fill_buf_area(buf, layout[1], " ", theme.data());

            Turbo.render(area, buf, &mut state.turbo, ctx)?;

            if state.error_dlg.active() {
                let layout_error = layout_middle(
                    layout[1],
                    Constraint::Percentage(19),
                    Constraint::Percentage(19),
                    Constraint::Length(2),
                    Constraint::Length(2),
                );
                let err = MsgDialog::new().styles(theme.msg_dialog_style());
                err.render(layout_error, buf, &mut state.error_dlg);
            }

            let el = t0.elapsed().unwrap_or(Duration::from_nanos(0));
            state.status.status(1, format!("R {:.0?}", el).to_string());

            let status = StatusLine::new()
                .layout([
                    Constraint::Fill(1),
                    Constraint::Length(8),
                    Constraint::Length(8),
                ])
                .styles(theme.statusline_style());
            status.render(layout[2], buf, &mut state.status);

            Ok(())
        }
    }

    /// Calculate the middle Rect inside a given area.
    fn layout_middle(
        area: Rect,
        left: Constraint,
        right: Constraint,
        top: Constraint,
        bottom: Constraint,
    ) -> Rect {
        let h_layout = Layout::horizontal([
            left, //
            Constraint::Fill(1),
            right,
        ])
        .split(area);
        let v_layout = Layout::vertical([
            top, //
            Constraint::Fill(1),
            bottom,
        ])
        .split(h_layout[1]);
        v_layout[1]
    }

    impl AppState<GlobalState, TurboEvent, Error> for SceneryState {
        fn init(&mut self, ctx: &mut AppContext<'_>) -> Result<(), Error> {
            ctx.focus = Some(FocusBuilder::build_for(&self.turbo));
            self.turbo.init(ctx)?;
            self.status.status(0, "Ctrl-Q to quit.");
            Ok(())
        }

        fn event(
            &mut self,
            event: &TurboEvent,
            ctx: &mut rat_salsa::AppContext<'_, GlobalState, TurboEvent, Error>,
        ) -> Result<Control<TurboEvent>, Error> {
            let t0 = SystemTime::now();

            let mut r = match event {
                TurboEvent::Event(event) => {
                    let mut r = match &event {
                        ct_event!(resized) => Control::Changed,
                        ct_event!(key press CONTROL-'q') => Control::Quit,
                        ct_event!(key press ALT-'x') => Control::Quit,
                        _ => Control::Continue,
                    };

                    r = r.or_else(|| {
                        if self.error_dlg.active() {
                            self.error_dlg.handle(&event, Dialog).into()
                        } else {
                            Control::Continue
                        }
                    });

                    r = r.or_else(|| {
                        ctx.focus = Some(FocusBuilder::rebuild_for(&self.turbo, ctx.focus.take()));
                        let f = ctx.focus_mut().handle(event, Regular);
                        ctx.queue(f);
                        Control::Continue
                    });

                    r
                }
                TurboEvent::Message(s) => {
                    self.error_dlg.append(s.as_str());
                    Control::Changed
                }
                TurboEvent::Status(n, s) => {
                    self.status.status(*n, s);
                    Control::Changed
                }
            };

            r = r.or_else_try(|| self.turbo.event(&event, ctx))?;

            let el = t0.elapsed().unwrap_or(Duration::from_nanos(0));
            self.status.status(2, format!("H {:.0?}", el).to_string());

            Ok(r)
        }

        fn error(
            &self,
            event: Error,
            _ctx: &mut AppContext<'_>,
        ) -> Result<Control<TurboEvent>, Error> {
            self.error_dlg.append(format!("{:?}", &*event).as_str());
            Ok(Control::Changed)
        }
    }
}

pub mod turbo {
    use crate::{GlobalState, RenderContext, TurboEvent};
    use anyhow::Error;
    use rat_salsa::{AppState, AppWidget, Control};
    use rat_widget::event::{ct_event, try_flow, HandleEvent, MenuOutcome, Popup};
    use rat_widget::focus::{FocusBuilder, FocusFlag, HasFocus};
    use rat_widget::menu::{
        MenuBuilder, MenuStructure, Menubar, MenubarState, PopupConstraint, PopupMenu,
        PopupMenuState,
    };
    use rat_widget::popup::Placement;
    use rat_widget::shadow::{Shadow, ShadowDirection};
    use ratatui::buffer::Buffer;
    use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
    use ratatui::style::{Style, Stylize};
    use ratatui::widgets::{Block, StatefulWidget};

    #[derive(Debug)]
    pub(crate) struct Turbo;

    #[derive(Debug)]
    pub struct TurboState {
        pub menu: MenubarState,
        pub menu_environment: PopupMenuState,
    }

    #[derive(Debug)]
    struct Menu;

    impl<'a> MenuStructure<'a> for Menu {
        fn menus(&'a self, menu: &mut MenuBuilder<'a>) {
            menu.item_parsed("_File")
                .item_parsed("_Edit")
                .item_parsed("_Search")
                .item_parsed("_Run")
                .item_parsed("_Compile")
                .item_parsed("_Debug")
                .item_parsed("_Tools")
                .item_parsed("_Options")
                .item_parsed("_Window")
                .item_parsed("_Help")
                .disabled(true);
        }

        fn submenu(&'a self, n: usize, submenu: &mut MenuBuilder<'a>) {
            match n {
                0 => {
                    submenu
                        .item_parsed("_New")
                        .item_parsed("_Open...|F3")
                        .item_parsed("_Save|F2")
                        .item_parsed("Save _as...")
                        .item_parsed("Save a_ll")
                        .item_parsed("\\___")
                        .item_parsed("_Change dir...")
                        .item_parsed("_Print")
                        .item_parsed("P_rinter setup...")
                        .item_parsed("_DOS shell")
                        .item_parsed("E_xit|ALt+X");
                }
                1 => {
                    submenu
                        .item_parsed("Undo|Alt+BkSp")
                        .item_parsed("Redo")
                        .disabled(true)
                        .item_parsed("\\___")
                        .item_parsed("Cu_t|Shift+Del")
                        .item_parsed("_Copy|Ctrl+Ins")
                        .item_parsed("_Paste|Shift+Ins")
                        .item_parsed("C_lear|Ctrl+Del")
                        .item_parsed("\\___")
                        .item_parsed("_Show clipboard");
                }
                2 => {
                    submenu
                        .item_parsed("_Find...")
                        .item_parsed("_Replace...")
                        .item_parsed("_Search again")
                        .item_parsed("\\___")
                        .item_parsed("_Go to line number...")
                        .item_parsed("Show last compiler error")
                        .item_parsed("Find _error...")
                        .item_parsed("Find _procedure");
                }
                3 => {
                    submenu
                        .item_parsed("_Run")
                        .item_parsed("_Step over")
                        .item_parsed("_Trace into")
                        .item_parsed("_Goto cursor")
                        .item_parsed("_Program reset")
                        .item_parsed("P_arameters...");
                }
                4 => {
                    submenu
                        .item_parsed("_Compile|Alt+F9")
                        .item_parsed("_Make|F9")
                        .item_parsed("_Build")
                        .item_parsed("\\___")
                        .item_parsed("_Destination|Disk")
                        .item_parsed("_Primary file...")
                        .item_parsed("C_lear primary file")
                        .item_parsed("\\___")
                        .item_parsed("_Information...");
                }
                5 => {
                    submenu
                        .item_parsed("_Breakpoints...")
                        .item_parsed("_Call stack|Ctrl+F3")
                        .item_parsed("_Register")
                        .item_parsed("_Watch")
                        .item_parsed("_Output")
                        .item_parsed("_User screen|Alt+F5")
                        .item_parsed("\\___")
                        .item_parsed("_Evaluate/modify...|Ctrl+F4")
                        .item_parsed("_Add watch...|Ctrl+F7")
                        .item_parsed("Add break_point");
                }
                6 => {
                    submenu
                        .item_parsed("_Messages")
                        .item_parsed("Go to _next|Alt+F8")
                        .disabled(true)
                        .item_parsed("Go to _previous|Alt+F7")
                        .disabled(true)
                        .item_parsed("\\___")
                        .item_parsed("_Grep")
                        .item_parsed("Clear/Refresh screen DOS")
                        .item_parsed("About TPWDB");
                }
                7 => {
                    submenu
                        .item_parsed("_Compiler...")
                        .item_parsed("_Memory sizes...")
                        .item_parsed("_Linker...")
                        .item_parsed("De_bugger...")
                        .item_parsed("_Directories...")
                        .item_parsed("_Tools...")
                        .item_parsed("\\___")
                        .item_parsed("_Environment|⏵")
                        .item_parsed("_Open...")
                        .item_parsed("_Save|")
                        .item_parsed("Save _as...");
                }
                8 => {
                    submenu
                        .item_parsed("_Tile")
                        .item_parsed("C_ascade")
                        .item_parsed("Cl_ose all")
                        .item_parsed("_Refresh display")
                        .item_parsed("\\___")
                        .item_parsed("_Size/Move|Ctrl+F5")
                        .item_parsed("_Zoom|F5")
                        .item_parsed("_Next|F6")
                        .item_parsed("_Previous|Shift+F6")
                        .item_parsed("_Close|Alt+F3")
                        .item_parsed("\\___")
                        .item_parsed("_List...|Alt+0");
                }
                9 => {
                    submenu
                        .item_parsed("_Contents")
                        .item_parsed("_Index|Shift+F1")
                        .item_parsed("_Topic search|Ctrl+F1")
                        .item_parsed("_Previous topic|Alt+F1")
                        .item_parsed("Using _help")
                        .item_parsed("_Files...")
                        .item_parsed("\\___")
                        .item_parsed("Compiler _directives")
                        .item_parsed("_Reserved words")
                        .item_parsed("Standard _units")
                        .item_parsed("Turbo Pascal _Language")
                        .item_parsed("_Error messages")
                        .item_parsed("\\___")
                        .item_parsed("_About...");
                }
                _ => {}
            }
        }
    }

    impl Default for TurboState {
        fn default() -> Self {
            Self {
                menu: Default::default(),
                menu_environment: Default::default(),
            }
        }
    }

    impl AppWidget<GlobalState, TurboEvent, Error> for Turbo {
        type State = TurboState;

        fn render(
            &self,
            area: Rect,
            buf: &mut Buffer,
            state: &mut Self::State,
            ctx: &mut RenderContext<'_>,
        ) -> Result<(), Error> {
            let theme = ctx.g.theme.clone();
            // TODO: repaint_mask

            let r = Layout::new(
                Direction::Vertical,
                [
                    Constraint::Length(1),
                    Constraint::Fill(1),
                    Constraint::Length(1),
                ],
            )
            .split(area);

            let (menubar, popup) = Menubar::new(&Menu)
                .styles(theme.menu_style())
                .title("  ")
                .popup_placement(Placement::Below)
                .popup_block(Block::bordered().style(theme.menu_style().style))
                .into_widgets();
            menubar.render(r[0], buf, &mut state.menu);
            popup.render(r[0], buf, &mut state.menu);

            if state.menu.popup.is_active() {
                Shadow::new()
                    .direction(ShadowDirection::BottomRight)
                    .style(Style::new().dark_gray().on_black())
                    .render(state.menu.popup.popup.area, buf, &mut ());
            }

            if state.menu_environment.is_active() {
                let area = state
                    .menu
                    .popup
                    .item_areas
                    .get(6)
                    .copied()
                    .unwrap_or_default();

                PopupMenu::new()
                    .styles(theme.menu_style())
                    .item_parsed("_Preferences...")
                    .item_parsed("_Editor...")
                    .item_parsed("_Mouse...")
                    .item_parsed("_Startup...")
                    .item_parsed("_Colors...")
                    .constraint(PopupConstraint::Right(Alignment::Left, area))
                    .y_offset(-1)
                    .block(Block::bordered())
                    .render(Rect::default(), buf, &mut state.menu_environment);

                Shadow::new().style(Style::new().on_black()).render(
                    state.menu_environment.popup.area,
                    buf,
                    &mut (),
                );
            }

            Ok(())
        }
    }

    impl HasFocus for TurboState {
        fn build(&self, builder: &mut FocusBuilder) {
            builder.widget(&self.menu);
        }

        fn focus(&self) -> FocusFlag {
            unimplemented!("not in use, silent container")
        }

        fn area(&self) -> Rect {
            unimplemented!("not in use, silent container")
        }
    }

    impl AppState<GlobalState, TurboEvent, Error> for TurboState {
        fn init(
            &mut self,
            ctx: &mut rat_salsa::AppContext<'_, GlobalState, TurboEvent, Error>,
        ) -> Result<(), Error> {
            ctx.focus().first();
            Ok(())
        }

        fn event(
            &mut self,
            event: &TurboEvent,
            ctx: &mut rat_salsa::AppContext<'_, GlobalState, TurboEvent, Error>,
        ) -> Result<Control<TurboEvent>, Error> {
            let r = match event {
                TurboEvent::Event(event) => {
                    if self.menu.selected() == (Some(7), Some(6)) {
                        try_flow!(match event {
                            ct_event!(keycode press Right) => {
                                self.menu_environment.set_active(true);
                                Control::Changed
                            }
                            _ => Control::Continue,
                        });
                    }
                    if self.menu_environment.is_active() {
                        try_flow!(match self.menu_environment.handle(event, Popup) {
                            MenuOutcome::Activated(_) => {
                                self.menu.popup.set_active(false);
                                Control::Changed
                            }
                            r => r.into(),
                        });
                    } else {
                        try_flow!({
                            let rr = self.menu.handle(event, Popup);
                            match rr {
                                MenuOutcome::MenuActivated(0, 9) => Control::Quit,
                                MenuOutcome::MenuActivated(7, 6) => {
                                    // reactivate menu
                                    self.menu.popup.set_active(true);
                                    self.menu_environment.set_active(true);
                                    Control::Changed
                                }
                                MenuOutcome::MenuActivated(6, 0) => {
                                    for _ in 0..50 {
                                        ctx.queue(Control::Event(TurboEvent::Message(
                                            "Hello!".into(),
                                        )));
                                    }
                                    Control::Changed
                                }
                                MenuOutcome::Selected(_) => {
                                    self.menu_environment.set_active(false);
                                    Control::Changed
                                }
                                v => v.into(),
                            }
                        });
                    }

                    Control::Continue
                }
                _ => Control::Continue,
            };

            Ok(r)
        }
    }
}

fn setup_logging() -> Result<(), Error> {
    if let Some(cache) = dirs::cache_dir() {
        let log_path = cache.join("rat-salsa");
        if !log_path.exists() {
            fs::create_dir_all(&log_path)?;
        }

        let log_file = log_path.join("minimal.log");
        _ = fs::remove_file(&log_file);
        fern::Dispatch::new()
            .format(|out, message, _record| {
                out.finish(format_args!("{}", message)) //
            })
            .level(log::LevelFilter::Debug)
            .chain(fern::log_file(&log_file)?)
            .apply()?;
    }
    Ok(())
}

#[allow(dead_code)]
pub mod theme {
    use rat_theme2::{Contrast, Scheme};
    use rat_widget::button::ButtonStyle;
    use rat_widget::file_dialog::FileDialogStyle;
    use rat_widget::line_number::LineNumberStyle;
    use rat_widget::list::ListStyle;
    use rat_widget::menu::MenuStyle;
    use rat_widget::msgdialog::MsgDialogStyle;
    use rat_widget::popup::PopupStyle;
    use rat_widget::scrolled::{ScrollStyle, ScrollSymbols};
    use rat_widget::splitter::SplitStyle;
    use rat_widget::tabbed::TabbedStyle;
    use rat_widget::table::TableStyle;
    use rat_widget::text::TextStyle;
    use ratatui::style::Style;
    use ratatui::style::Stylize;
    use ratatui::widgets::Block;

    #[derive(Debug, Clone)]
    pub struct TurboTheme {
        s: Scheme,
        name: String,
    }

    impl TurboTheme {
        pub fn new(name: String, s: Scheme) -> Self {
            Self { s, name }
        }
    }

    impl TurboTheme {
        /// Some display name.
        pub fn name(&self) -> &str {
            &self.name
        }

        /// Hint at dark.
        pub fn dark_theme(&self) -> bool {
            false
        }

        /// The underlying scheme.
        pub fn scheme(&self) -> &Scheme {
            &self.s
        }

        /// Focus style
        pub fn focus(&self) -> Style {
            let fg = self.s.black[0];
            let bg = self.s.primary[2];
            Style::default().fg(fg).bg(bg)
        }

        /// Selection style
        pub fn select(&self) -> Style {
            let fg = self.s.black[0];
            let bg = self.s.secondary[1];
            Style::default().fg(fg).bg(bg)
        }

        /// Text field style.
        pub fn text_input(&self) -> Style {
            Style::default().fg(self.s.black[0]).bg(self.s.gray[3])
        }

        /// Focused text field style.
        pub fn text_focus(&self) -> Style {
            let fg = self.s.black[0];
            let bg = self.s.primary[0];
            Style::default().fg(fg).bg(bg)
        }

        /// Text selection style.
        pub fn text_select(&self) -> Style {
            let fg = self.s.black[0];
            let bg = self.s.secondary[0];
            Style::default().fg(fg).bg(bg)
        }

        /// Data display style. Used for lists, tables, ...
        pub fn data(&self) -> Style {
            Style::default().fg(self.s.white[0]).bg(self.s.deepblue[0])
        }

        /// Background for dialogs.
        pub fn dialog_style(&self) -> Style {
            Style::default().fg(self.s.black[0]).bg(self.s.gray[3])
        }

        /// Style for the status line.
        pub fn status_style(&self) -> Style {
            Style::default().fg(self.s.black[0]).bg(self.s.gray[3])
        }

        /// Style for LineNumbers.
        pub fn line_nr_style(&self) -> LineNumberStyle {
            LineNumberStyle {
                style: self.data().fg(self.s.gray[0]),
                cursor: Some(self.text_select()),
                ..LineNumberStyle::default()
            }
        }

        /// Complete TextAreaStyle
        pub fn textarea_style(&self) -> TextStyle {
            TextStyle {
                style: self.data(),
                focus: Some(self.focus()),
                select: Some(self.text_select()),
                ..Default::default()
            }
        }

        /// Complete TextInputStyle
        pub fn input_style(&self) -> TextStyle {
            TextStyle {
                style: self.text_input(),
                focus: Some(self.text_focus()),
                select: Some(self.text_select()),
                invalid: Some(Style::default().bg(self.s.red[3])),
                ..Default::default()
            }
        }

        /// Complete MenuStyle
        pub fn menu_style(&self) -> MenuStyle {
            MenuStyle {
                style: self.dialog_style(),
                title: Some(Style::default().fg(self.s.black[0]).bg(self.s.gray[3])),
                select: Some(self.select()),
                focus: Some(self.focus()),
                highlight: Some(Style::default().fg(self.s.red[2])),
                disabled: Some(Style::default().fg(self.s.black[3])),
                right: Some(Style::default().italic()),
                popup: PopupStyle {
                    style: self.dialog_style(),
                    border_style: Some(Style::default().fg(self.s.black[3])),
                    ..Default::default()
                },
                ..Default::default()
            }
        }

        /// Complete FTableStyle
        pub fn table_style(&self) -> TableStyle {
            TableStyle {
                style: self.data(),
                select_row: Some(self.select()),
                show_row_focus: true,
                focus_style: Some(self.focus()),
                ..Default::default()
            }
        }

        /// Complete ListStyle
        pub fn list_style(&self) -> ListStyle {
            ListStyle {
                style: self.data(),
                select: Some(self.select()),
                focus: Some(self.focus()),
                ..Default::default()
            }
        }

        /// Complete ButtonStyle
        pub fn button_style(&self) -> ButtonStyle {
            ButtonStyle {
                style: self.s.primary(0, Contrast::Normal),
                focus: Some(self.s.primary(3, Contrast::High)),
                armed: Some(Style::default().fg(self.s.black[0]).bg(self.s.secondary[0])),
                ..Default::default()
            }
        }

        /// Complete ScrolledStyle
        pub fn scroll_style(&self) -> ScrollStyle {
            let style = Style::default().fg(self.s.black[3]);
            let arrow_style = Style::default().fg(self.s.black[3]).bg(self.s.secondary[0]);
            ScrollStyle {
                thumb_style: Some(style),
                track_style: Some(style),
                min_style: Some(style),
                begin_style: Some(arrow_style),
                end_style: Some(arrow_style),
                vertical: Some(ScrollSymbols {
                    track: "\u{2592}",
                    thumb: "█",
                    begin: "▲",
                    end: "▼",
                    min: "\u{2591}",
                }),
                horizontal: Some(ScrollSymbols {
                    track: "\u{2592}",
                    thumb: "█",
                    begin: "◄",
                    end: "►",
                    min: "\u{2591}",
                }),
                ..Default::default()
            }
        }

        /// Complete Split style
        pub fn split_style(&self) -> SplitStyle {
            let style = Style::default().fg(self.s.gray[0]).bg(self.s.black[1]);
            let arrow_style = Style::default().fg(self.s.secondary[0]).bg(self.s.black[1]);
            SplitStyle {
                style,
                arrow_style: Some(arrow_style),
                drag_style: Some(self.focus()),
                ..Default::default()
            }
        }

        /// Complete Tabbed style
        pub fn tabbed_style(&self) -> TabbedStyle {
            let style = Style::default().fg(self.s.gray[0]).bg(self.s.black[1]);
            TabbedStyle {
                style,
                tab: Some(self.s.gray(1, Contrast::Normal)),
                select: Some(self.s.gray(3, Contrast::Normal)),
                focus: Some(self.focus()),
                ..Default::default()
            }
        }

        /// Complete StatusLineStyle for a StatusLine with 3 indicator fields.
        /// This is what I need for the
        /// [minimal](https://github.com/thscharler/rat-salsa/blob/master/examples/minimal.rs)
        /// example, which shows timings for Render/Event/Action.
        pub fn statusline_style(&self) -> Vec<Style> {
            let s = &self.s;
            vec![
                self.status_style(),
                s.blue(3, Contrast::Normal),
                s.blue(2, Contrast::Normal),
                s.blue(1, Contrast::Normal),
            ]
        }

        /// Complete MsgDialogStyle.
        pub fn msg_dialog_style(&self) -> MsgDialogStyle {
            MsgDialogStyle {
                style: self.dialog_style(),
                button: Some(self.button_style()),
                scroll: Some(self.scroll_style()),
                ..Default::default()
            }
        }

        pub fn file_dialog_style(&self) -> FileDialogStyle {
            FileDialogStyle {
                style: self.dialog_style(),
                list: Some(self.list_style()),
                roots: Some(self.list_style()),
                text: Some(self.input_style()),
                button: Some(self.button_style()),
                block: Some(Block::bordered()),
                ..Default::default()
            }
        }
    }
}
