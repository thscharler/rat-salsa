fn main() {
    unreachable!("not functional")
}

//
//
// use anyhow::anyhow;
// use crossterm::cursor::{DisableBlinking, EnableBlinking, SetCursorStyle};
// use crossterm::event::{
//     DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste, EnableMouseCapture, KeyCode,
//     KeyEvent, KeyEventKind, KeyModifiers,
// };
// use crossterm::terminal::{
//     disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
// };
// use crossterm::ExecutableCommand;
// use rat_event::{FocusKeys, HandleEvent};
// use rat_scrolled::scrolled::{Scrolled, ScrolledState};
// use ratatui::backend::CrosstermBackend;
// use ratatui::buffer::Buffer;
// use ratatui::layout::{Constraint, Direction, Layout, Rect};
// use ratatui::style::{Style, Stylize};
// use ratatui::widgets::StatefulWidget;
// use ratatui::{Frame, Terminal};
// use std::fs;
// use std::io::{stdout, Stdout};
// use std::time::Duration;
// use tui_tree_widget::TreeItem;
//
// mod adapter;
//
// fn main() -> Result<(), anyhow::Error> {
//     setup_logging()?;
//
//     let mut data = Data {};
//     let mut state = State {
//         tree1: Default::default(),
//     };
//
//     run_ui(&mut data, &mut state)
// }
//
// fn setup_logging() -> Result<(), anyhow::Error> {
//     _ = fs::remove_file("log.log");
//     fern::Dispatch::new()
//         .format(|out, message, record| {
//             out.finish(format_args!(
//                 "[{} {} {}]\n",
//                 record.level(),
//                 record.target(),
//                 message
//             ))
//         })
//         .level(log::LevelFilter::Debug)
//         .chain(fern::log_file("log.log")?)
//         .apply()?;
//     Ok(())
// }
//
// struct Data {}
//
// struct State {
//     pub(crate) tree1: ScrolledState<TreeSState<i32>>,
// }
//
// fn run_ui(data: &mut Data, state: &mut State) -> Result<(), anyhow::Error> {
//     stdout().execute(EnterAlternateScreen)?;
//     stdout().execute(EnableMouseCapture)?;
//     stdout().execute(EnableBlinking)?;
//     stdout().execute(SetCursorStyle::BlinkingBar)?;
//     stdout().execute(EnableBracketedPaste)?;
//     enable_raw_mode()?;
//
//     let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
//     terminal.clear()?;
//
//     repaint_ui(&mut terminal, data, state)?;
//
//     let r = 'l: loop {
//         let o = match crossterm::event::poll(Duration::from_millis(10)) {
//             Ok(true) => {
//                 let event = match crossterm::event::read() {
//                     Ok(v) => v,
//                     Err(e) => break 'l Err(anyhow!(e)),
//                 };
//                 match handle_event(event, data, state) {
//                     Ok(v) => v,
//                     Err(e) => break 'l Err(e),
//                 }
//             }
//             Ok(false) => continue,
//             Err(e) => break 'l Err(anyhow!(e)),
//         };
//
//         match o {
//             Outcome::Changed => {
//                 match repaint_ui(&mut terminal, data, state) {
//                     Ok(_) => {}
//                     Err(e) => break 'l Err(e),
//                 };
//             }
//             _ => {
//                 // noop
//             }
//         }
//     };
//
//     disable_raw_mode()?;
//     stdout().execute(DisableBracketedPaste)?;
//     stdout().execute(SetCursorStyle::DefaultUserShape)?;
//     stdout().execute(DisableBlinking)?;
//     stdout().execute(DisableMouseCapture)?;
//     stdout().execute(LeaveAlternateScreen)?;
//
//     r
// }
//
// fn repaint_ui(
//     terminal: &mut Terminal<CrosstermBackend<Stdout>>,
//     data: &mut Data,
//     state: &mut State,
// ) -> Result<(), anyhow::Error> {
//     terminal.hide_cursor()?;
//
//     _ = terminal.draw(|frame| {
//         _ = repaint_tui(frame, data, state);
//     });
//
//     Ok(())
// }
//
// fn repaint_tui(
//     frame: &mut Frame<'_>,
//     data: &mut Data,
//     state: &mut State,
// ) -> Result<(), anyhow::Error> {
//     let area = frame.size();
//     let buffer = frame.buffer_mut();
//
//     repaint_lists(area, buffer, data, state)
// }
//
// fn handle_event(
//     event: crossterm::event::Event,
//     data: &mut Data,
//     state: &mut State,
// ) -> Result<Outcome, anyhow::Error> {
//     use crossterm::event::Event;
//     match event {
//         Event::Key(KeyEvent {
//             code: KeyCode::Char('q'),
//             modifiers: KeyModifiers::CONTROL,
//             kind: KeyEventKind::Press,
//             ..
//         }) => {
//             return Err(anyhow!("quit"));
//         }
//         Event::Resize(_, _) => return Ok(Outcome::Changed),
//         _ => {}
//     }
//
//     let r = handle_lists(&event, data, state)?;
//
//     Ok(r)
// }
//
// fn repaint_lists(
//     area: Rect,
//     buf: &mut Buffer,
//     _data: &mut Data,
//     state: &mut State,
// ) -> Result<(), anyhow::Error> {
//     let l_columns = Layout::new(
//         Direction::Horizontal,
//         [
//             Constraint::Fill(2),
//             Constraint::Fill(1),
//             Constraint::Fill(2),
//             Constraint::Fill(1),
//         ],
//     )
//     .split(Rect::new(area.x, area.y + 1, area.width, area.height - 1));
//
//     let tree = Scrolled::new(
//         TreeS::new(vec![
//             TreeItem::new(
//                 1,
//                 "1",
//                 vec![
//                     TreeItem::new_leaf(10, "10"),
//                     TreeItem::new_leaf(11, "11"),
//                     TreeItem::new_leaf(12, "12"),
//                     TreeItem::new_leaf(13, "13"),
//                     TreeItem::new_leaf(14, "14"),
//                     TreeItem::new_leaf(15, "15"),
//                     TreeItem::new_leaf(16, "16"),
//                     TreeItem::new_leaf(17, "17"),
//                     TreeItem::new_leaf(18, "18"),
//                     TreeItem::new_leaf(19, "19"),
//                 ],
//             )?,
//             TreeItem::new(
//                 2,
//                 "2",
//                 vec![
//                     TreeItem::new_leaf(20, "20"),
//                     TreeItem::new_leaf(21, "21"),
//                     TreeItem::new_leaf(22, "22"),
//                     TreeItem::new_leaf(23, "23"),
//                     TreeItem::new_leaf(24, "24"),
//                     TreeItem::new_leaf(25, "25"),
//                     TreeItem::new_leaf(26, "26"),
//                     TreeItem::new_leaf(27, "27"),
//                     TreeItem::new_leaf(28, "28"),
//                     TreeItem::new_leaf(29, "29"),
//                 ],
//             )?,
//             TreeItem::new(
//                 3,
//                 "3",
//                 vec![
//                     TreeItem::new_leaf(30, "30"),
//                     TreeItem::new_leaf(31, "31"),
//                     TreeItem::new_leaf(32, "32"),
//                     TreeItem::new_leaf(33, "33"),
//                     TreeItem::new_leaf(34, "34"),
//                     TreeItem::new_leaf(35, "35"),
//                     TreeItem::new_leaf(36, "36"),
//                     TreeItem::new_leaf(37, "37"),
//                     TreeItem::new_leaf(38, "38"),
//                     TreeItem::new_leaf(39, "39"),
//                 ],
//             )?,
//             TreeItem::new(
//                 4,
//                 "4",
//                 vec![
//                     TreeItem::new_leaf(40, "40"),
//                     TreeItem::new_leaf(41, "41"),
//                     TreeItem::new_leaf(42, "42"),
//                     TreeItem::new_leaf(43, "43"),
//                     TreeItem::new_leaf(44, "44"),
//                     TreeItem::new_leaf(45, "45"),
//                     TreeItem::new_leaf(46, "46"),
//                     TreeItem::new_leaf(47, "47"),
//                     TreeItem::new_leaf(48, "48"),
//                     TreeItem::new_leaf(49, "49"),
//                 ],
//             )?,
//             TreeItem::new(
//                 5,
//                 "5",
//                 vec![
//                     TreeItem::new_leaf(50, "50"),
//                     TreeItem::new_leaf(51, "51"),
//                     TreeItem::new_leaf(52, "52"),
//                     TreeItem::new_leaf(53, "53"),
//                     TreeItem::new_leaf(54, "54"),
//                     TreeItem::new_leaf(55, "55"),
//                     TreeItem::new_leaf(56, "56"),
//                     TreeItem::new_leaf(57, "57"),
//                     TreeItem::new_leaf(58, "58"),
//                     TreeItem::new_leaf(59, "59"),
//                 ],
//             )?,
//         ])
//         .highlight_style(Style::default().on_red()),
//     );
//
//     // for i in 0..60 {
//     //     state.tree1.widget.widget.open(vec![i / 10]);
//     //     state.tree1.widget.widget.open(vec![i / 10, i]);
//     // }
//
//     StatefulWidget::render(tree, l_columns[0], buf, &mut state.tree1);
//
//     Ok(())
// }
//
// fn handle_lists(
//     event: &crossterm::event::Event,
//     _data: &mut Data,
//     state: &mut State,
// ) -> Result<Outcome, anyhow::Error> {
//     match HandleEvent::handle(&mut state.tree1, event, FocusKeys) {
//         Outcome::Continue => {}
//         r => return Ok(r),
//     };
//
//     Ok(Outcome::Continue)
// }
