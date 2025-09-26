//!
//! Copies the menu-structure of Turbo Pascal 7.0.
//!
//! and mimics the style. Turns out the base16 theme doesn't
//! look too bad.
//!
//!

use crate::app::Scenery;
use crate::theme::TurboTheme;
use anyhow::Error;
use rat_dialog::{DialogStack, WindowControl};
use rat_salsa::poll::PollCrossterm;
use rat_salsa::{Control, RunConfig, SalsaAppContext, SalsaContext, run_tui};
use rat_theme3::palettes::BASE16;
use ratatui::layout::Rect;
use std::fs;
use std::path::PathBuf;
use try_as::traits::TryAsRef;

type TurboResult = Result<Control<TurboEvent>, Error>;
type TurboDialogResult = Result<WindowControl<TurboEvent>, Error>;

fn main() -> Result<(), Error> {
    setup_logging()?;

    let config = TurboConfig::default();
    let theme = TurboTheme::new("Base16".into(), BASE16);
    let mut global = Global::new(config, theme);
    let mut state = Scenery::default();

    run_tui(
        app::init,
        app::render,
        app::event,
        app::error,
        &mut global,
        &mut state,
        RunConfig::default()? //
            .poll(PollCrossterm),
    )?;

    Ok(())
}

/// Globally accessible data/state.
#[derive(Debug)]
pub struct Global {
    ctx: SalsaAppContext<TurboEvent, Error>,
    pub cfg: TurboConfig,
    pub theme: TurboTheme,
    pub dialogs: DialogStack<TurboEvent, Global, Error>,
    pub area: Rect,
}

impl SalsaContext<TurboEvent, Error> for Global {
    fn set_salsa_ctx(&mut self, app_ctx: SalsaAppContext<TurboEvent, Error>) {
        self.ctx = app_ctx;
    }

    fn salsa_ctx(&self) -> &SalsaAppContext<TurboEvent, Error> {
        &self.ctx
    }
}

impl Global {
    pub fn new(cfg: TurboConfig, theme: TurboTheme) -> Self {
        Self {
            ctx: Default::default(),
            cfg,
            theme,
            dialogs: Default::default(),
            area: Default::default(),
        }
    }
}

/// Configuration.
#[derive(Debug, Default)]
pub struct TurboConfig {}

/// Application wide messages.
#[derive(Debug)]
pub enum TurboEvent {
    Event(crossterm::event::Event),
    Message(String),
    Status(usize, String),
    NoOp,

    NewDialog,
    OpenDialog,
    SaveAsDialog,
}

impl From<crossterm::event::Event> for TurboEvent {
    fn from(value: crossterm::event::Event) -> Self {
        Self::Event(value)
    }
}

impl TryAsRef<crossterm::event::Event> for TurboEvent {
    fn try_as_ref(&self) -> Option<&crossterm::event::Event> {
        match self {
            TurboEvent::Event(e) => Some(e),
            _ => None,
        }
    }
}

impl<'a> TryFrom<&'a TurboEvent> for &'a crossterm::event::Event {
    type Error = ();

    fn try_from(value: &'a TurboEvent) -> Result<Self, Self::Error> {
        match value {
            TurboEvent::Event(event) => Ok(event),
            _ => Err(()),
        }
    }
}

pub mod app {
    use crate::turbo::Turbo;
    use crate::{Global, TurboDialogResult, TurboEvent, TurboResult, menu, turbo};
    use anyhow::Error;
    use rat_dialog::{WindowControl, handle_dialog_stack};
    use rat_event::{Dialog, Outcome, break_flow, try_flow};
    use rat_salsa::{Control, SalsaContext};
    use rat_widget::event::{HandleEvent, ct_event};
    use rat_widget::file_dialog::FileDialogState;
    use rat_widget::focus::FocusBuilder;
    use rat_widget::layout::layout_middle;
    use rat_widget::menu::{MenubarState, PopupMenuState};
    use rat_widget::msgdialog::{MsgDialog, MsgDialogState};
    use rat_widget::statusline::{StatusLine, StatusLineState};
    use rat_widget::util::fill_buf_area;
    use ratatui::buffer::Buffer;
    use ratatui::layout::{Constraint, Layout, Rect};
    use ratatui::widgets::StatefulWidget;
    use std::any::Any;
    use std::time::{Duration, SystemTime};

    #[derive(Debug, Default)]
    pub struct Scenery {
        pub turbo: Turbo,
        pub menu: MenubarState,
        pub menu_environment: PopupMenuState,
        pub status: StatusLineState,
    }

    pub fn init(state: &mut Scenery, ctx: &mut Global) -> Result<(), Error> {
        build_focus(state, ctx);
        turbo::init(&mut state.turbo, ctx)?;
        state.status.status(0, "Ctrl-Q to quit.");
        Ok(())
    }

    fn handle_focus(event: &TurboEvent, state: &mut Scenery, ctx: &mut Global) -> TurboResult {
        if let TurboEvent::Event(event) = event {
            build_focus(state, ctx);
            ctx.handle_focus(event);
        }
        Ok(Control::Continue)
    }

    fn build_focus(state: &mut Scenery, ctx: &mut Global) {
        let mut builder = FocusBuilder::new(ctx.take_focus());
        builder.widget(&state.menu);

        for i in 0..ctx.dialogs.len() {
            if let Some(s) = ctx.dialogs.try_get::<FileDialogState>(i) {
                builder.widget(&*s);
            }
            if let Some(s) = ctx.dialogs.try_get::<MsgDialogState>(i) {
                builder.widget(&*s);
            }
        }

        ctx.set_focus(builder.build());
    }

    pub fn render(
        area: Rect,
        buf: &mut Buffer,
        state: &mut Scenery,
        ctx: &mut Global,
    ) -> Result<(), Error> {
        let t0 = SystemTime::now();

        let layout = Layout::vertical([
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .split(area);

        ctx.area = layout[1];

        fill_buf_area(buf, layout[1], " ", ctx.theme.data());

        let menu_popup = menu::render(layout[0], buf, ctx, state);
        turbo::render(area, buf, &mut state.turbo, ctx)?;
        ctx.dialogs.clone().render(layout[1], buf, ctx);
        render_status(t0, layout[2], buf, state, ctx);
        menu::render_popup(menu_popup, layout[0], buf, ctx, state);

        Ok(())
    }

    fn render_status(
        t0: SystemTime,
        area: Rect,
        buf: &mut Buffer,
        state: &mut Scenery,
        ctx: &mut Global,
    ) {
        let el = t0.elapsed().unwrap_or(Duration::from_nanos(0));
        state.status.status(1, format!("R {:.0?}", el).to_string());

        let status = StatusLine::new()
            .layout([
                Constraint::Fill(1),
                Constraint::Length(8),
                Constraint::Length(8),
            ])
            .styles(ctx.theme.statusline_style());
        status.render(area, buf, &mut state.status);
    }

    #[allow(unused_variables)]
    pub fn event(
        event: &TurboEvent,
        state: &mut Scenery,
        ctx: &mut Global,
    ) -> Result<Control<TurboEvent>, Error> {
        let t0 = SystemTime::now();

        let r = 'b: {
            break_flow!('b: handle_super_keys(event, state, ctx)?);
            break_flow!('b: handle_scenery(event, state, ctx)?);
            break_flow!('b: {
                match handle_dialog_stack(ctx.dialogs.clone(), event, ctx)? {
                    WindowControl::Continue => Control::Continue,
                    WindowControl::Unchanged => Control::Unchanged,
                    WindowControl::Changed => Control::Changed,
                    WindowControl::Event(e) => Control::Event(e),
                    WindowControl::Close(e) => {
                        ctx.queue_event(e);
                        Control::Changed
                    }
                }
            });
            break_flow!('b: handle_focus(event, state, ctx)?);
            break_flow!('b: menu::event(event, state, ctx)?);
            break_flow!('b: turbo::event(event, &mut state.turbo, ctx)?);
            Control::Continue
        };

        state.status.status(
            2,
            format!("H {:.0?}", t0.elapsed().expect("t0")).to_string(),
        );

        Ok(r)
    }

    fn handle_super_keys(
        event: &TurboEvent,
        _state: &mut Scenery,
        _ctx: &mut Global,
    ) -> TurboResult {
        match event {
            TurboEvent::Event(event) => match &event {
                ct_event!(resized) => Ok(Control::Changed),
                ct_event!(key press CONTROL-'q') => Ok(Control::Quit),
                ct_event!(key press ALT-'x') => Ok(Control::Quit),
                _ => Ok(Control::Continue),
            },
            _ => Ok(Control::Continue),
        }
    }

    fn handle_scenery(event: &TurboEvent, state: &mut Scenery, ctx: &mut Global) -> TurboResult {
        match event {
            TurboEvent::Message(s) => {
                show_message(s.as_str(), ctx) //
            }
            TurboEvent::Status(n, s) => {
                state.status.status(*n, s);
                Ok(Control::Changed)
            }
            _ => Ok(Control::Continue),
        }
    }

    pub fn error(
        event: Error,
        _state: &mut Scenery,
        ctx: &mut Global,
    ) -> Result<Control<TurboEvent>, Error> {
        show_message(format!("{:?}", &*event).as_str(), ctx)
    }

    fn show_message(msg: &str, ctx: &mut Global) -> TurboResult {
        if let Some(n) = ctx.dialogs.first::<MsgDialogState>() {
            let v = ctx.dialogs.get::<MsgDialogState>(n);
            v.append(msg);
        } else {
            let state = MsgDialogState::new_active("Information", msg);
            ctx.focus().future(&state);

            ctx.dialogs.push(render_msg_dlg, handle_msg_dlg, state)
        }

        Ok(Control::Changed)
    }

    fn render_msg_dlg(area: Rect, buf: &mut Buffer, state: &mut dyn Any, ctx: &mut Global) {
        let state = state.downcast_mut().expect("msgdialog-state");

        let area = layout_middle(
            area,
            Constraint::Percentage(19),
            Constraint::Percentage(19),
            Constraint::Length(2),
            Constraint::Length(2),
        );

        MsgDialog::new()
            .styles(ctx.theme.msg_dialog_style())
            .render(area, buf, state);
    }

    fn handle_msg_dlg(
        event: &TurboEvent,
        state: &mut dyn Any,
        _ctx: &mut Global,
    ) -> TurboDialogResult {
        if let TurboEvent::Event(event) = event {
            let state = state
                .downcast_mut::<MsgDialogState>()
                .expect("msgdialog-state");

            try_flow!(match state.handle(event, Dialog) {
                Outcome::Changed => {
                    if !state.active() {
                        WindowControl::Close(TurboEvent::NoOp)
                    } else {
                        WindowControl::Changed
                    }
                }
                r => r.into(),
            });
        }

        Ok(WindowControl::Continue)
    }
}

pub mod menu {
    use crate::app::Scenery;
    use crate::{Global, TurboEvent};
    use anyhow::Error;
    use rat_event::{HandleEvent, Popup, ct_event, try_flow};
    use rat_salsa::{Control, SalsaContext};
    use rat_widget::event::MenuOutcome;
    use rat_widget::menu::{
        MenuBuilder, MenuStructure, Menubar, MenubarPopup, PopupConstraint, PopupMenu,
    };
    use rat_widget::popup::Placement;
    use rat_widget::shadow::{Shadow, ShadowDirection};
    use ratatui::buffer::Buffer;
    use ratatui::layout::{Alignment, Rect};
    use ratatui::style::{Style, Stylize};
    use ratatui::widgets::{Block, StatefulWidget};

    #[derive(Debug)]
    pub struct Menu;

    pub fn render(
        area: Rect,
        buf: &mut Buffer,
        ctx: &mut Global,
        state: &mut Scenery,
    ) -> MenubarPopup<'static> {
        let (menubar, popup) = Menubar::new(&Menu)
            .styles(ctx.theme.menu_style())
            .title("  ")
            .popup_placement(Placement::Below)
            .popup_block(Block::bordered().style(ctx.theme.menu_style().style))
            .into_widgets();
        menubar.render(area, buf, &mut state.menu);

        popup
    }

    pub fn render_popup(
        popup: MenubarPopup<'static>,
        area: Rect,
        buf: &mut Buffer,
        ctx: &mut Global,
        state: &mut Scenery,
    ) {
        popup.render(area, buf, &mut state.menu);
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
                .styles(ctx.theme.menu_style())
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
    }

    pub fn event(
        event: &TurboEvent,
        state: &mut Scenery,
        ctx: &mut Global,
    ) -> Result<Control<TurboEvent>, Error> {
        match event {
            TurboEvent::Event(event) => {
                if state.menu.selected() == (Some(7), Some(6)) {
                    try_flow!(match event {
                        ct_event!(keycode press Right) => {
                            state.menu_environment.set_active(true);
                            Control::Changed
                        }
                        _ => Control::Continue,
                    });
                }
                if state.menu_environment.is_active() {
                    try_flow!(match state.menu_environment.handle(event, Popup) {
                        MenuOutcome::Activated(_) => {
                            state.menu.popup.set_active(false);
                            Control::Changed
                        }
                        r => r.into(),
                    });
                } else {
                    try_flow!({
                        let rr = state.menu.handle(event, Popup);
                        match rr {
                            MenuOutcome::MenuActivated(0, 0) => {
                                Control::Event(TurboEvent::NewDialog)
                            }
                            MenuOutcome::MenuActivated(0, 1) => {
                                Control::Event(TurboEvent::OpenDialog)
                            }
                            MenuOutcome::MenuActivated(0, 3) => {
                                Control::Event(TurboEvent::SaveAsDialog)
                            }
                            MenuOutcome::MenuActivated(0, 9) => Control::Quit,
                            MenuOutcome::MenuActivated(7, 6) => {
                                // reactivate menu
                                state.menu.popup.set_active(true);
                                state.menu_environment.set_active(true);
                                Control::Changed
                            }
                            MenuOutcome::MenuActivated(6, 0) => {
                                for _ in 0..50 {
                                    ctx.queue(Control::Event(TurboEvent::Message("Hello!".into())));
                                }
                                Control::Changed
                            }
                            MenuOutcome::Selected(_) => {
                                state.menu_environment.set_active(false);
                                Control::Changed
                            }
                            v => v.into(),
                        }
                    });
                }
            }
            _ => {}
        }

        Ok(Control::Continue)
    }

    impl MenuStructure<'static> for Menu {
        fn menus(&'static self, menu: &mut MenuBuilder<'static>) {
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

        fn submenu(&'static self, n: usize, submenu: &mut MenuBuilder<'static>) {
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
}

pub mod turbo {
    use crate::{Global, TurboDialogResult, TurboEvent};
    use anyhow::Error;
    use rat_dialog::WindowControl;
    use rat_event::{Dialog, Outcome};
    use rat_salsa::{Control, SalsaContext};
    use rat_widget::event::{FileOutcome, HandleEvent, try_flow};
    use rat_widget::file_dialog::{FileDialog, FileDialogState};
    use rat_widget::layout::layout_middle;
    use rat_widget::text::HasScreenCursor;
    use ratatui::buffer::Buffer;
    use ratatui::layout::{Constraint, Rect};
    use ratatui::widgets::StatefulWidget;
    use std::any::Any;
    use std::path::PathBuf;

    #[derive(Debug, Default)]
    pub struct Turbo {}

    pub fn render(
        _area: Rect,
        _buf: &mut Buffer,
        _state: &mut Turbo,
        _ctx: &mut Global,
    ) -> Result<(), Error> {
        Ok(())
    }

    pub fn init(_state: &mut Turbo, ctx: &mut Global) -> Result<(), Error> {
        ctx.focus().first();
        Ok(())
    }

    pub fn event(
        event: &TurboEvent,
        _state: &mut Turbo,
        ctx: &mut Global,
    ) -> Result<Control<TurboEvent>, Error> {
        match event {
            TurboEvent::NewDialog => try_flow!(show_new(ctx)?),
            TurboEvent::OpenDialog => try_flow!(show_open(ctx)?),
            TurboEvent::SaveAsDialog => try_flow!(show_save_as(ctx)?),
            _ => {}
        }

        Ok(Control::Continue)
    }

    fn show_new(ctx: &mut Global) -> Result<Control<TurboEvent>, Error> {
        let mut state = FileDialogState::new();

        state.save_dialog_ext(PathBuf::from("."), "", "pas")?;
        ctx.focus().future(&state);

        ctx.dialogs.push(render_new_dlg, handle_new_dlg, state);

        Ok(Control::Changed)
    }

    fn render_new_dlg(area: Rect, buf: &mut Buffer, state: &mut dyn Any, ctx: &mut Global) {
        let state = state
            .downcast_mut::<FileDialogState>()
            .expect("dialog-state");

        let area = layout_middle(
            area,
            Constraint::Percentage(19),
            Constraint::Percentage(19),
            Constraint::Length(2),
            Constraint::Length(2),
        );

        FileDialog::new()
            .styles(ctx.theme.file_dialog_style())
            .render(area, buf, state);

        ctx.set_screen_cursor(state.screen_cursor());
    }

    fn handle_new_dlg(
        event: &TurboEvent,
        state: &mut dyn Any,
        _ctx: &mut Global,
    ) -> TurboDialogResult {
        let state = state
            .downcast_mut::<FileDialogState>()
            .expect("dialog-state");

        match event {
            TurboEvent::Event(event) => {
                try_flow!(match state.handle(event, Dialog)? {
                    FileOutcome::Cancel => {
                        WindowControl::Close(TurboEvent::NoOp) //
                    }
                    FileOutcome::Ok(f) => {
                        WindowControl::Close(
                            TurboEvent::Message(format!("New file {:?}", f)), //
                        )
                    }
                    r => Outcome::from(r).into(),
                });
            }
            _ => {}
        }

        Ok(WindowControl::Continue)
    }

    fn show_open(ctx: &mut Global) -> Result<Control<TurboEvent>, Error> {
        let mut state = FileDialogState::new();

        state.open_dialog(PathBuf::from("."))?;
        ctx.focus().future(&state);

        ctx.dialogs.push(render_open_dlg, handle_open_dlg, state);
        Ok(Control::Changed)
    }

    fn render_open_dlg(area: Rect, buf: &mut Buffer, state: &mut dyn Any, ctx: &mut Global) {
        let state = state
            .downcast_mut::<FileDialogState>()
            .expect("dialog-state");

        let area = layout_middle(
            area,
            Constraint::Percentage(19),
            Constraint::Percentage(19),
            Constraint::Length(2),
            Constraint::Length(2),
        );

        FileDialog::new()
            .styles(ctx.theme.file_dialog_style())
            .render(area, buf, state);

        ctx.set_screen_cursor(state.screen_cursor());
    }

    fn handle_open_dlg(
        event: &TurboEvent,
        state: &mut dyn Any,
        _ctx: &mut Global,
    ) -> TurboDialogResult {
        let state = state
            .downcast_mut::<FileDialogState>()
            .expect("dialog-state");

        match event {
            TurboEvent::Event(event) => {
                try_flow!(match state.handle(event, Dialog)? {
                    FileOutcome::Cancel => {
                        WindowControl::Close(TurboEvent::NoOp) //
                    }
                    FileOutcome::Ok(f) => {
                        WindowControl::Close(
                            TurboEvent::Status(0, format!("Open {:?}", f)), //
                        )
                    }
                    r => r.into(),
                });
            }
            _ => {}
        };

        Ok(WindowControl::Continue)
    }

    fn show_save_as(ctx: &mut Global) -> Result<Control<TurboEvent>, Error> {
        let mut state = FileDialogState::new();
        state.save_dialog_ext(PathBuf::from("."), "", "pas")?;

        ctx.focus().future(&state);
        ctx.dialogs
            .push(render_save_as_dlg, handle_save_as_dlg, state);

        Ok(Control::Changed)
    }

    fn handle_save_as_dlg(
        event: &TurboEvent,
        state: &mut dyn Any,
        _ctx: &mut Global,
    ) -> TurboDialogResult {
        let state = state
            .downcast_mut::<FileDialogState>()
            .expect("dialog-state");

        match event {
            TurboEvent::Event(event) => {
                try_flow!(match state.handle(event, Dialog)? {
                    FileOutcome::Cancel => {
                        WindowControl::Close(TurboEvent::NoOp) //
                    }
                    FileOutcome::Ok(f) => {
                        WindowControl::Close(
                            TurboEvent::Status(0, format!("Save as {:?}", f)), //
                        )
                    }
                    r => r.into(),
                });
            }
            _ => {}
        }

        Ok(WindowControl::Continue)
    }

    fn render_save_as_dlg(area: Rect, buf: &mut Buffer, state: &mut dyn Any, ctx: &mut Global) {
        let state = state
            .downcast_mut::<FileDialogState>()
            .expect("dialog-state");

        let area = layout_middle(
            area,
            Constraint::Percentage(19),
            Constraint::Percentage(19),
            Constraint::Length(2),
            Constraint::Length(2),
        );

        FileDialog::new()
            .styles(ctx.theme.file_dialog_style())
            .render(area, buf, state);

        ctx.set_screen_cursor(state.screen_cursor());
    }
}

fn setup_logging() -> Result<(), Error> {
    if let Some(cache) = dirs::cache_dir() {
        let log_file = if cfg!(debug_assertions) {
            PathBuf::from("log.log")
        } else {
            let log_path = cache.join("rat-salsa");
            if !log_path.exists() {
                fs::create_dir_all(&log_path)?;
            }
            log_path.join("turbo.log")
        };

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
    use rat_theme3::{Contrast, Palette};
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
    use ratatui::style::{Style, Stylize};

    #[derive(Debug, Clone)]
    pub struct TurboTheme {
        s: Palette,
        name: String,
    }

    impl TurboTheme {
        pub fn new(name: String, s: Palette) -> Self {
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
        pub fn scheme(&self) -> &Palette {
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
            Style::default().fg(self.s.white[3]).bg(self.s.deepblue[0])
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
                style: self.dialog_style(),
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
                style: self.s.secondary(0, Contrast::High),
                focus: Some(self.s.primary(0, Contrast::High)),
                armed: Some(Style::default().fg(self.s.black[0]).bg(self.s.secondary[3])),
                hover: Some(Style::default().fg(self.s.black[0]).bg(self.s.primary[0])),
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
                ..Default::default()
            }
        }
    }
}
