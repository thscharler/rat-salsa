#![allow(dead_code)]

use crate::mini_salsa::endless_scroll::{EndlessScroll, EndlessScrollState};
use crate::mini_salsa::{MiniSalsaState, mock_init, run_ui, setup_logging};
use rat_event::{HandleEvent, Regular, ct_event, event_flow, try_flow};
use rat_focus::{Focus, FocusBuilder, HasFocus, Navigation, match_focus};
use rat_menu::event::MenuOutcome;
use rat_menu::menuline::{MenuLine, MenuLineState};
use rat_reloc::RelocatableState;
use rat_scrolled::Scroll;
use rat_theme4::{StyleName, WidgetStyle};
use rat_widget::event::{Outcome, TabbedOutcome};
use rat_widget::list::selection::RowSelection;
use rat_widget::list::{List, ListState};
use rat_widget::statusline::StatusLineState;
use rat_widget::tabbed::{TabPlacement, TabType, Tabbed, TabbedState};
use ratatui_core::buffer::Buffer;
use ratatui_core::layout::{Constraint, Layout, Rect};
use ratatui_core::style::{Style, Stylize};
use ratatui_core::symbols::border;
use ratatui_core::text::Line;
use ratatui_core::widgets::{StatefulWidget, Widget};
use ratatui_crossterm::crossterm::event::Event;
use ratatui_widgets::block::Block;
use ratatui_widgets::borders::BorderType;

mod mini_salsa;

fn main() -> Result<(), anyhow::Error> {
    setup_logging()?;

    let mut state = State {
        border_type: None,
        placement: TabPlacement::default(),
        style: TabType::default(),
        close: false,
        nav_mode: 0,
        tabbed: TabbedState::default(),
        tabs_0: Default::default(),
        tabs_1: Default::default(),
        tabs_2: Default::default(),
        menu: MenuLineState::default(),
        status: StatusLineState::default(),
    };
    state.menu.focus.set(true);

    run_ui("tabbed1", mock_init, event, render, &mut state)
}

struct Data {}

struct State {
    border_type: Option<(BorderType, border::Set<'static>)>,
    placement: TabPlacement,
    style: TabType,
    close: bool,
    nav_mode: u8,

    tabbed: TabbedState,

    tabs_0: ListState<RowSelection>,
    tabs_1: EndlessScrollState,
    tabs_2: EndlessScrollState,

    menu: MenuLineState,
    status: StatusLineState,
}

fn render(
    buf: &mut Buffer,
    area: Rect,
    ctx: &mut MiniSalsaState,
    state: &mut State,
) -> Result<(), anyhow::Error> {
    let l1 = Layout::vertical([
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .split(area);

    let l2 = Layout::horizontal([
        Constraint::Length(25),
        Constraint::Fill(1),
        Constraint::Length(15),
    ])
    .split(l1[1]);

    let mut tab = Tabbed::new()
        .styles(ctx.theme.style(WidgetStyle::TABBED))
        .tab_type(state.style)
        .placement(state.placement);
    if state.close {
        tab = tab.closeable(true);
    }
    if let Some(border_type) = state.border_type {
        tab = tab.block(
            Block::bordered()
                .border_type(border_type.0)
                .border_set(border_type.1),
        );
    }
    tab = tab.tabs(["Issues", "Numbers", "More numbers"]);
    tab.render(l2[1], buf, &mut state.tabbed);

    if state.tabbed.selected() == Some(0) {
        List::<RowSelection>::new(LIST)
            .scroll(Scroll::new())
            .styles(ctx.theme.style(WidgetStyle::LIST))
            .render(state.tabbed.widget_area, buf, &mut state.tabs_0)
    } else {
        // clear any areas from previous renders.
        state.tabs_0.relocate_hidden();
    }
    if state.tabbed.selected() == Some(1) {
        EndlessScroll::new()
            .max(2024)
            .style(ctx.theme.p.bluegreen(0))
            .focus_style(ctx.theme.style_style(Style::FOCUS))
            .v_scroll(Scroll::new().styles(ctx.theme.style(WidgetStyle::SCROLL)))
            .render(state.tabbed.widget_area, buf, &mut state.tabs_1)
    } else {
        // clear any areas from previous renders.
        state.tabs_1.relocate_hidden();
    }
    if state.tabbed.selected() == Some(2) {
        EndlessScroll::new()
            .max(2024)
            .style(ctx.theme.p.cyan(0))
            .focus_style(ctx.theme.style_style(Style::FOCUS))
            .v_scroll(Scroll::new().styles(ctx.theme.style(WidgetStyle::SCROLL)))
            .render(state.tabbed.widget_area, buf, &mut state.tabs_2)
    } else {
        // clear any areas from previous renders.
        state.tabs_2.relocate_hidden();
    }

    let mut area = Rect::new(l2[0].x, l2[0].y, l2[0].width, 1);

    Line::from("F1: close").yellow().render(area, buf);
    area.y += 1;
    Line::from("F2: type").yellow().render(area, buf);
    area.y += 1;
    Line::from("F3: alignment").yellow().render(area, buf);
    area.y += 1;
    Line::from("F5: border").yellow().render(area, buf);
    area.y += 1;
    Line::from("F11: nav-mode").yellow().render(area, buf);
    area.y += 1;
    Line::from("F12: key-nav").yellow().render(area, buf);

    MenuLine::new()
        .title("||||")
        .item_parsed("_Quit")
        .title_style(Style::default().black().on_yellow())
        .style(Style::default().black().on_dark_gray())
        .render(l1[3], buf, &mut state.menu);

    Ok(())
}

fn focus(state: &State) -> Focus {
    match state.nav_mode {
        0 => {
            // default tabbed behaviour
            let mut fb = FocusBuilder::default();
            fb.widget(&state.tabbed);
            match state.tabbed.selected {
                Some(0) => _ = fb.widget(&state.tabs_0),
                Some(1) => _ = fb.widget(&state.tabs_1),
                Some(2) => _ = fb.widget(&state.tabs_2),
                _ => {}
            }
            fb.widget(&state.menu);
            let f = fb.build();
            f.enable_log();
            f
        }
        1 => {
            // tabbed as regular widget
            let mut fb = FocusBuilder::default();
            fb.widget_navigate(&state.tabbed, Navigation::Regular);
            match state.tabbed.selected {
                Some(0) => _ = fb.widget(&state.tabs_0),
                Some(1) => _ = fb.widget(&state.tabs_1),
                Some(2) => _ = fb.widget(&state.tabs_2),
                _ => {}
            }
            fb.widget(&state.menu);
            let f = fb.build();
            f.enable_log();
            f
        }
        2 => {
            // tabs follow focus
            let mut fb = FocusBuilder::default();
            fb.widget(&state.tabbed);
            fb.widget(&state.tabs_0);
            fb.widget(&state.tabs_1);
            fb.widget(&state.tabs_2);
            fb.widget(&state.menu);
            let f = fb.build();
            f.enable_log();
            f
        }
        _ => unimplemented!(),
    }
}

fn event(
    event: &Event,
    ctx: &mut MiniSalsaState,
    state: &mut State,
) -> Result<Outcome, anyhow::Error> {
    ctx.focus = Some(focus(state));
    ctx.handle_focus(event);

    // debug!("{:#?}", ctx.focus);

    match event {
        ct_event!(keycode press F(1)) => event_flow!({
            state.close = !state.close;
            Outcome::Changed
        }),
        ct_event!(keycode press F(2)) => event_flow!({
            state.style = match state.style {
                TabType::Glued => TabType::Attached,
                TabType::Attached => TabType::Glued,
                _ => TabType::Glued,
            };
            Outcome::Changed
        }),
        ct_event!(keycode press SHIFT-F(2)) => event_flow!({
            state.style = match state.style {
                TabType::Glued => TabType::Attached,
                TabType::Attached => TabType::Glued,
                _ => TabType::Glued,
            };
            Outcome::Changed
        }),
        ct_event!(keycode press F(3)) => event_flow!({
            state.placement = match state.placement {
                TabPlacement::Top => TabPlacement::Right,
                TabPlacement::Right => TabPlacement::Bottom,
                TabPlacement::Bottom => TabPlacement::Left,
                TabPlacement::Left => TabPlacement::Top,
            };
            Outcome::Changed
        }),
        ct_event!(keycode press SHIFT-F(3)) => event_flow!({
            state.placement = match state.placement {
                TabPlacement::Top => TabPlacement::Left,
                TabPlacement::Right => TabPlacement::Top,
                TabPlacement::Bottom => TabPlacement::Right,
                TabPlacement::Left => TabPlacement::Bottom,
            };
            Outcome::Changed
        }),
        ct_event!(keycode press F(5)) => event_flow!({
            state.border_type = match state.border_type {
                None => Some((BorderType::Plain, border::PLAIN)),
                Some((BorderType::Plain, border::PLAIN)) => {
                    Some((BorderType::Plain, border::ONE_EIGHTH_TALL))
                }
                Some((BorderType::Plain, border::ONE_EIGHTH_TALL)) => {
                    Some((BorderType::Plain, border::ONE_EIGHTH_WIDE))
                }
                Some((BorderType::Plain, border::ONE_EIGHTH_WIDE)) => {
                    Some((BorderType::Plain, border::PROPORTIONAL_WIDE))
                }
                Some((BorderType::Plain, border::PROPORTIONAL_WIDE)) => {
                    Some((BorderType::Plain, border::PROPORTIONAL_TALL))
                }
                Some((BorderType::Plain, border::PROPORTIONAL_TALL)) => {
                    Some((BorderType::Double, border::DOUBLE))
                }
                Some((BorderType::Double, border::DOUBLE)) => {
                    Some((BorderType::Rounded, border::ROUNDED))
                }
                Some((BorderType::Rounded, border::ROUNDED)) => {
                    Some((BorderType::Thick, border::THICK))
                }
                Some((BorderType::Thick, border::THICK)) => {
                    Some((BorderType::QuadrantInside, border::QUADRANT_INSIDE))
                }
                Some((BorderType::QuadrantInside, border::QUADRANT_INSIDE)) => {
                    Some((BorderType::QuadrantOutside, border::QUADRANT_OUTSIDE))
                }
                Some((BorderType::QuadrantOutside, border::QUADRANT_OUTSIDE)) => None,
                _ => Some((BorderType::Plain, border::PLAIN)),
            };
            Outcome::Changed
        }),
        ct_event!(keycode press SHIFT-F(5)) => event_flow!({
            state.border_type = match state.border_type {
                None => Some((BorderType::QuadrantOutside, border::QUADRANT_OUTSIDE)),
                Some((BorderType::Plain, border::PLAIN)) => None,
                Some((BorderType::Plain, border::ONE_EIGHTH_TALL)) => {
                    Some((BorderType::Plain, border::PLAIN))
                }
                Some((BorderType::Plain, border::ONE_EIGHTH_WIDE)) => {
                    Some((BorderType::Plain, border::ONE_EIGHTH_TALL))
                }
                Some((BorderType::Plain, border::PROPORTIONAL_WIDE)) => {
                    Some((BorderType::Plain, border::ONE_EIGHTH_WIDE))
                }
                Some((BorderType::Plain, border::PROPORTIONAL_TALL)) => {
                    Some((BorderType::Plain, border::PROPORTIONAL_WIDE))
                }
                Some((BorderType::Double, border::DOUBLE)) => {
                    Some((BorderType::Plain, border::PROPORTIONAL_TALL))
                }
                Some((BorderType::Rounded, border::ROUNDED)) => {
                    Some((BorderType::Double, border::DOUBLE))
                }
                Some((BorderType::Thick, border::THICK)) => {
                    Some((BorderType::Rounded, border::ROUNDED))
                }
                Some((BorderType::QuadrantInside, border::QUADRANT_INSIDE)) => {
                    Some((BorderType::Thick, border::THICK))
                }
                Some((BorderType::QuadrantOutside, border::QUADRANT_OUTSIDE)) => {
                    Some((BorderType::QuadrantInside, border::QUADRANT_INSIDE))
                }
                _ => Some((BorderType::Plain, border::PLAIN)),
            };
            Outcome::Changed
        }),
        ct_event!(keycode press F(10)) => event_flow!({
            state.nav_mode = (state.nav_mode + 1) % 3;
            Outcome::Changed
        }),
        ct_event!(keycode press F(12)) => event_flow!({
            focus(state).focus(&state.tabbed);
            Outcome::Changed
        }),
        _ => {}
    }

    if state.nav_mode == 2 {
        'x: {
            let sel = match_focus!(
                state.tabbed => break 'x,
                state.tabs_0 => Some(0),
                state.tabs_1 => Some(1),
                state.tabs_2 => Some(2),
                else => None
            );
            state.tabbed.select(sel)
        }
    }

    match HandleEvent::handle(&mut state.tabbed, event, Regular) {
        TabbedOutcome::Select(0) => event_flow!({
            if !state.tabbed.is_focused() {
                ctx.focus().future(&state.tabs_0)
            };
            Outcome::Changed
        }),
        TabbedOutcome::Select(1) => event_flow!({
            if !state.tabbed.is_focused() {
                ctx.focus().future(&state.tabs_1);
            }
            Outcome::Changed
        }),
        TabbedOutcome::Select(2) => event_flow!({
            if !state.tabbed.is_focused() {
                ctx.focus().future(&state.tabs_2);
            }
            Outcome::Changed
        }),
        r => event_flow!(r),
    }

    match state.tabbed.selected() {
        Some(0) => event_flow!(state.tabs_0.handle(event, Regular)),
        Some(1) => event_flow!(state.tabs_1.handle(event, Regular)),
        Some(2) => event_flow!(state.tabs_2.handle(event, Regular)),
        _ => {}
    }

    match state.menu.handle(event, Regular) {
        MenuOutcome::Activated(0) => event_flow!({
            ctx.quit = true;
            Outcome::Changed
        }),
        r => event_flow!(r),
    }

    Ok(Outcome::Continue)
}

static LIST: [&str; 28] = [
    "Advisory: Ratatui / Crossterm Version incompatibility (0.27 / 0.28 and beyond)
#1298 · joshka opened on Aug 6, 2024",
    "Examples may not be compatible with the latest release
#779 · joshka opened on Jan 10, 2024",
    "Ratatui's Vision
#1321 · orhun opened on Aug 13, 2024",
    "Convert the buffer type into a trait with the current buffer as one of it's implementations
Type: Enhancement
Status: Open.
#1450 In ratatui/ratatui;· giocri opened on Oct 25, 2024",
    "Canvas rendering issue when area is huge
Type: Bug
Status: Open.
#1449 In ratatui/ratatui;· JeromeSchmied opened on Oct 22, 2024",
    "Checkbox and Radio button widgets
Status: Design Needed
Type: Enhancement
Status: Open.
#1446 In ratatui/ratatui;· fujiapple852 opened on Oct 21, 2024",
    "Stylize the highlight_symbol in List Widget
    Effort: Good First Issue
Type: Enhancement
Status: Open.
#1443 In ratatui/ratatui;· aktaboot opened on Oct 21, 2024",
    "Implement Rect::resize similar to Rect::offset
Type: Enhancement
Status: Open.
#1440 In ratatui/ratatui;· kardwen opened on Oct 20, 2024",
    "Inline viewport should support an Terminal::insert_lines_before method.
Type: Enhancement
Status: Open.
#1426 In ratatui/ratatui;· nfachan opened on Oct 17, 2024",
    "Rendering Caching needs to be one level deeper
Priority: Low
Type: Bug
Status: Open.
#1405 In ratatui/ratatui;· joshka opened on Oct 6, 2024",
    "Support asserting TestBackend buffer with color
Effort: Good First Issue
Type: Enhancement
Status: Open.
#1402 In ratatui/ratatui;· orhun opened on Oct 5, 2024",
    "Introduce new way to compare visually output of layout tests
Type: Enhancement
Status: Open.
#1400 In ratatui/ratatui;· kdheepak opened on Oct 3, 2024",
    "Korean characters are not rendered correctly
Type: Bug
Status: Open.
#1396 In ratatui/ratatui;· sxyazi opened on Oct 2, 2024",
    "GraphType::Bar does not work when y-axis minimum bound is > 0
Type: Bug
Status: Open.
#1391 In ratatui/ratatui;· Yomguithereal opened on Sep 30, 2024",
    "Modularize Ratatui crate
Type: Enhancement
Type: RFC
Status: Open.
#1388 In ratatui/ratatui;· joshka opened on Sep 27, 2024",
    "Most examples don't work or look bad in macOS's Terminal.app
Type: Bug
Status: Open.
#1387 In ratatui/ratatui;· ccbrown opened on Sep 25, 2024",
    "Scrolling the list widget by page
Effort: Difficult
Status: Design Needed
Type: Enhancement
Status: Open.
#1370 In ratatui/ratatui;· francisdb opened on Sep 12, 2024",
    "insert_before dynamic height based on content
Type: Enhancement
Status: Open.
#1365 In ratatui/ratatui;· staehle opened on Sep 11, 2024",
    "Add support for Dash borders
Effort: Easy
Effort: Good First Issue
Type: Enhancement
Status: Open.
#1355 In ratatui/ratatui;· 0xfalafel opened on Sep 6, 2024",
    "dweatherstone
crossterm::style::Stylize is imported instead of ratatui::style::Stylize
Type: Bug
Status: Open.
#1347 In ratatui/ratatui;· orhun opened on Aug 28, 2024",
    "underline_color() on an input field makes the screen flicker from that line to the bottom
Status: Pending
Type: Bug
Status: Open.
#1346 In ratatui/ratatui;· mazzi opened on Aug 27, 2024",
    "Extremely high CPU usage of the terminal.draw method
Type: Bug
Status: Open.
#1338 In ratatui/ratatui;· gtema opened on Aug 23, 2024",
    "insert_before and emoji width empty space issue
Type: Bug
Status: Open.
#1332 In ratatui/ratatui;· Kongga666 opened on Aug 20, 2024",
    "Ratatui's Vision
Type: Enhancement
Status: Open.
#1321 In ratatui/ratatui;· orhun opened on Aug 13, 2024",
    "Feature request: Add feature flags for backends
Type: Enhancement
Status: Open.
#1319 In ratatui/ratatui;· nick42d opened on Aug 11, 2024",
    "Advisory: Ratatui / Crossterm Version incompatibility (0.27 / 0.28 and beyond)
Type: Documentation
Status: Open.
#1298 In ratatui/ratatui;· joshka opened on Aug 6, 2024",
    "Allow reversing Chart Legend direction
Type: Enhancement
Status: Open.
#1290 In ratatui/ratatui;· nullstalgia opened on Aug 5, 2024",
    "WidgetRef / StatefulWidgetRef tracking issue
Type: Documentation
Status: Open.
#1287 In ratatui/ratatui;· joshka opened on Aug 5, 2024",
];
