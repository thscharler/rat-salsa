use crate::util::configparser_ext::ConfigParserExt;
use crate::{Global, Scenery};
use anyhow::{Error, anyhow};
use configparser::ini::Ini;
use rat_theme4::palette::{ColorIdx, Colors, Palette};
use rat_theme4::theme::SalsaTheme;
use rat_theme4::{RatWidgetColor, themes};
use ratatui::prelude::{Color, Line};
use std::array;
use std::path::Path;

pub fn pal_aliases(pal: Palette) -> Vec<(Option<String>, String)> {
    pal.aliased
        .iter()
        .map(|(v, _)| (Some(v.to_string()), v.to_string()))
        .collect()
}

pub fn pal_choice(pal: Palette) -> Vec<(ColorIdx, Line<'static>)> {
    const COLOR_X_8: usize = Colors::LEN * 8 + 1;
    let pal_choice = array::from_fn::<_, COLOR_X_8, _>(|n| {
        if n == Colors::LEN * 8 {
            let c = Colors::None;
            let n = 0;
            (c, n)
        } else {
            let c = color_array()[n / 8];
            let n = n % 8;
            (c, n)
        }
    });
    pal_choice
        .iter()
        .map(|(c, n)| {
            (
                ColorIdx(*c, *n),
                Line::from(format!("{}-{}", c, n)).style(pal.style(*c, *n)),
            )
        })
        .collect::<Vec<_>>()
}

pub fn rat_widget_color_names() -> &'static [&'static str] {
    &[
        Color::LABEL_FG,
        Color::INPUT_BG,
        Color::INPUT_FOCUS_BG,
        Color::INPUT_SELECT_BG,
        Color::FOCUS_BG,
        Color::SELECT_BG,
        Color::DISABLED_BG,
        Color::INVALID_BG,
        //
        Color::TITLE_FG,
        Color::TITLE_BG,
        Color::HEADER_FG,
        Color::HEADER_BG,
        Color::FOOTER_FG,
        Color::FOOTER_BG,
        //
        Color::HOVER_BG,
        Color::BUTTON_BASE_BG,
        Color::KEY_BINDING_BG,
        Color::MENU_BASE_BG,
        Color::STATUS_BASE_BG,
        Color::SHADOW_BG,
        Color::WEEK_HEADER_FG,
        Color::MONTH_HEADER_FG,
        //
        Color::CONTAINER_BASE_BG,
        Color::CONTAINER_BORDER_FG,
        Color::CONTAINER_ARROW_FG,
        Color::DOCUMENT_BASE_BG,
        Color::DOCUMENT_BORDER_FG,
        Color::DOCUMENT_ARROW_FG,
        Color::POPUP_BASE_BG,
        Color::POPUP_BORDER_FG,
        Color::POPUP_ARROW_FG,
        Color::DIALOG_BASE_BG,
        Color::DIALOG_BORDER_FG,
        Color::DIALOG_ARROW_FG,
    ]
}

pub const fn color_array_no_text() -> [Colors; Colors::LEN - 2] {
    use rat_theme4::palette::Colors::*;
    [
        Primary, Secondary, White, Black, Gray, Red, Orange, Yellow, LimeGreen, Green, BlueGreen,
        Cyan, Blue, DeepBlue, Purple, Magenta, RedPink,
    ]
}

pub const fn color_array() -> [Colors; Colors::LEN] {
    use rat_theme4::palette::Colors::*;
    [
        TextLight, TextDark, Primary, Secondary, White, Black, Gray, Red, Orange, Yellow,
        LimeGreen, Green, BlueGreen, Cyan, Blue, DeepBlue, Purple, Magenta, RedPink,
    ]
}

pub fn save_patch(path: &Path, state: &mut Scenery, ctx: &mut Global) -> Result<(), Error> {
    let mut ff = Ini::new_std();
    ff.set_text("palette-patch", "name", state.edit.name());
    ff.set_text("palette-patch", "docs", state.edit.docs.text());

    let aliased = state.edit.aliased_for(ctx.cfg.extra_aliased_vec());
    for (c, c_idx) in aliased {
        ff.set_val("reference", c.as_ref(), c_idx);
    }

    ff.write_std(path)?;

    Ok(())
}

pub fn load_patch(path: &Path, state: &mut Scenery, ctx: &mut Global) -> Result<(), Error> {
    let mut ff = Ini::new_std();
    match ff.load(path) {
        Ok(_) => {}
        Err(e) => return Err(anyhow!(e)),
    };

    let extra = ctx.cfg.extra_aliased_vec();
    for (n, s) in state.edit.color_ext.iter_mut() {
        if extra.contains(n) {
            let c_idx = ff.parse_val("reference", n, ColorIdx::default());
            s.set_value(c_idx);
        }
    }

    ctx.show_theme = create_edit_theme(state);

    Ok(())
}

// todo: save-patch

pub fn export_pal_to_patch(
    path: &Path,
    state: &mut Scenery,
    ctx: &mut Global,
) -> Result<(), Error> {
    use std::fs::File;
    use std::io::Write;

    let mut wr = File::create(path)?;
    writeln!(
        wr,
        "use rat_theme4::palette::{{ColorIdx, Colors, Palette}};"
    )?;
    writeln!(wr, "")?;
    writeln!(wr, "/// Patch for {}", state.edit.name())?;
    for l in state.edit.docs.text().lines() {
        writeln!(wr, "/// {}", l)?;
    }
    writeln!(wr, "")?;
    writeln!(wr, "pub fn patch(pal: &mut Palette) {{",)?;
    writeln!(
        wr,
        "    if pal.name.as_ref() == \"{}\" {{",
        state.edit.name()
    )?;
    let aliased = state.edit.aliased_for(ctx.cfg.extra_aliased_vec());
    for (n, c) in aliased {
        writeln!(
            wr,
            "        pal.add_aliased({:?}, ColorIdx(Colors::{:?}, {:?}));",
            n, c.0, c.1
        )?;
    }
    writeln!(wr, "    }}")?;
    writeln!(wr, "}}")?;
    writeln!(wr, "")?;

    Ok(())
}

pub fn export_pal_to_rs(path: &Path, state: &mut Scenery, ctx: &mut Global) -> Result<(), Error> {
    use std::fs::File;
    use std::io::Write;

    let c32 = Palette::color_to_u32;

    let mut wr = File::create(path)?;
    writeln!(wr, "use std::borrow::Cow;")?;
    writeln!(wr, "use crate::palette::{{Colors, Palette, define_alias}};")?;
    writeln!(wr, "")?;
    writeln!(wr, "/// {}", state.edit.name())?;
    for l in state.edit.docs.text().lines() {
        writeln!(wr, "/// {}", l)?;
    }
    writeln!(
        wr,
        "const DARKNESS: u8 = {};",
        state.edit.dark.value::<u8>().unwrap_or(64)
    )?;
    writeln!(wr, "")?;
    writeln!(
        wr,
        "pub const {}: Palette = Palette {{",
        state.edit.const_name(),
    )?;
    writeln!(wr, "    name: Cow::Borrowed(\"{}\"), ", state.edit.name())?;
    writeln!(wr, "")?;
    writeln!(wr, "    color: [")?;
    for c in [Colors::TextLight, Colors::TextDark] {
        let c0 = state.edit.color[c as usize].0.value();
        let c3 = state.edit.color[c as usize].3.value();
        writeln!(
            wr,
            "        Palette::interpolate2({:#08x}, {:#08x}, 0x0, 0x0),",
            c32(c0),
            c32(c3)
        )?;
    }
    for c in color_array_no_text() {
        let c0 = state.edit.color[c as usize].0.value();
        let c3 = state.edit.color[c as usize].3.value();
        writeln!(
            wr,
            "        Palette::interpolate({:#08x}, {:#08x}, DARKNESS),",
            c32(c0),
            c32(c3)
        )?;
    }
    writeln!(wr, "    ],")?;
    writeln!(wr, "    // must be sorted!")?;
    writeln!(wr, "    aliased: Cow::Borrowed(&[")?;
    let aliased = state.edit.aliased_for(ctx.cfg.aliased_vec());
    for (n, c) in aliased {
        writeln!(
            wr,
            "        define_alias({:?}, Colors::{:?}, {:?}),",
            n, c.0, c.1
        )?;
    }
    writeln!(wr, "    ]),")?;
    writeln!(wr, "}};")?;
    writeln!(wr, "")?;

    Ok(())
}

pub fn save_pal(path: &Path, state: &mut Scenery, ctx: &mut Global) -> Result<(), Error> {
    state.file_path = Some(path.into());

    let mut ff = Ini::new_std();
    ff.set_text("palette", "name", state.edit.name());
    ff.set_text("palette", "docs", state.edit.docs.text());
    ff.set_val(
        "palette",
        "dark",
        state.edit.dark.value::<u8>().unwrap_or(63),
    );
    for c in color_array() {
        ff.set_array(
            "color",
            &c.to_string(),
            [
                state.edit.color[c as usize].0.value(),
                state.edit.color[c as usize].3.value(),
            ],
        );
    }

    let aliased = state.edit.aliased_for(ctx.cfg.aliased_vec());
    for (c, c_idx) in aliased {
        ff.set_val("reference", c.as_ref(), c_idx);
    }

    ff.write_std(path)?;

    Ok(())
}

pub fn new_pal(state: &mut Scenery, _ctx: &mut Global) -> Result<(), Error> {
    state.file_path = None;

    state.edit.name.set_value("pal.name");
    _ = state.edit.dark.set_value(64);

    for c in color_array() {
        state.edit.color[c as usize].0.set_value(Color::default());
        state.edit.color[c as usize].3.set_value(Color::default());
    }
    for (_, s) in state.edit.color_ext.iter_mut() {
        s.set_value(ColorIdx(Colors::default(), 0));
    }

    state
        .detail
        .show
        .readability
        .bg_color
        .set_value(ColorIdx(Colors::default(), 0));

    Ok(())
}

pub fn use_base46(state: &mut Scenery, _ctx: &mut Global) -> Result<(), Error> {
    if let Some(v) = state.detail.foreign.color("white") {
        state.edit.color[Colors::TextLight as usize].0.set_value(v);
        state.edit.color[Colors::TextLight as usize].3.set_value(v);
    }
    if let Some(v) = state.detail.foreign.color("darker_black") {
        state.edit.color[Colors::TextDark as usize].0.set_value(v);
        state.edit.color[Colors::TextDark as usize].3.set_value(v);
    }

    if let Some(v) = state.detail.foreign.color("white") {
        state.edit.color[Colors::White as usize].0.set_value(v);
        state.edit.color[Colors::White as usize].3.set_value(v);
    }
    if let Some(v) = state.detail.foreign.color("grey") {
        state.edit.color[Colors::Gray as usize].0.set_value(v);
    }
    if let Some(v) = state.detail.foreign.color("light_grey") {
        state.edit.color[Colors::Gray as usize].3.set_value(v);
    }
    if let Some(v) = state.detail.foreign.color("darker_black") {
        state.edit.color[Colors::Black as usize].0.set_value(v);
    }
    if let Some(v) = state.detail.foreign.color("black2") {
        state.edit.color[Colors::Black as usize].3.set_value(v);
    }
    if let Some(v) = state.detail.foreign.color("red") {
        state.edit.color[Colors::Red as usize].0.set_value(v);
        state.edit.color[Colors::Red as usize].3.set_value(v);
    }
    if let Some(v) = state.detail.foreign.color("orange") {
        state.edit.color[Colors::Orange as usize].0.set_value(v);
        state.edit.color[Colors::Orange as usize].3.set_value(v);
    }
    if let Some(v) = state.detail.foreign.color("yellow") {
        state.edit.color[Colors::Yellow as usize].0.set_value(v);
        state.edit.color[Colors::Yellow as usize].3.set_value(v);
    }
    if let Some(v) = state.detail.foreign.color("vibrant_green") {
        state.edit.color[Colors::LimeGreen as usize].0.set_value(v);
        state.edit.color[Colors::LimeGreen as usize].3.set_value(v);
    }
    if let Some(v) = state.detail.foreign.color("green") {
        state.edit.color[Colors::Green as usize].0.set_value(v);
        state.edit.color[Colors::Green as usize].3.set_value(v);
    }
    if let Some(v) = state.detail.foreign.color("teal") {
        state.edit.color[Colors::BlueGreen as usize].0.set_value(v);
        state.edit.color[Colors::BlueGreen as usize].3.set_value(v);
    }
    if let Some(v) = state.detail.foreign.color("cyan") {
        state.edit.color[Colors::Cyan as usize].0.set_value(v);
        state.edit.color[Colors::Cyan as usize].3.set_value(v);
    }
    if let Some(v) = state.detail.foreign.color("blue") {
        state.edit.color[Colors::Blue as usize].0.set_value(v);
        state.edit.color[Colors::Blue as usize].3.set_value(v);
    }
    if let Some(v) = state.detail.foreign.color("nord_blue") {
        state.edit.color[Colors::DeepBlue as usize].0.set_value(v);
        state.edit.color[Colors::DeepBlue as usize].3.set_value(v);
    }
    if let Some(v) = state.detail.foreign.color("dark_purple") {
        state.edit.color[Colors::Purple as usize].0.set_value(v);
        state.edit.color[Colors::Purple as usize].3.set_value(v);
    }
    if let Some(v) = state.detail.foreign.color("pink") {
        state.edit.color[Colors::Magenta as usize].0.set_value(v);
        state.edit.color[Colors::Magenta as usize].3.set_value(v);
    }
    if let Some(v) = state.detail.foreign.color("baby_pink") {
        state.edit.color[Colors::RedPink as usize].0.set_value(v);
        state.edit.color[Colors::RedPink as usize].3.set_value(v);
    }
    Ok(())
}

pub fn load_pal(path: &Path, state: &mut Scenery, ctx: &mut Global) -> Result<(), Error> {
    state.file_path = Some(path.into());

    let mut ff = Ini::new_std();
    match ff.load(path) {
        Ok(_) => {}
        Err(e) => return Err(anyhow!(e)),
    };

    state
        .edit
        .name
        .set_value(ff.get_text("palette", "name", ""));
    state
        .edit
        .docs
        .set_value(ff.get_text("palette", "docs", ""));
    _ = state
        .edit
        .dark
        .set_value(ff.parse_val::<u8, _>("palette", "dark", 63));
    for c in color_array() {
        let ccc = ff.parse_array::<2, _, _>("color", &c.to_string(), Color::default());
        state.edit.color[c as usize].0.set_value(ccc[0]);
        state.edit.color[c as usize].3.set_value(ccc[1]);
    }
    for (n, s) in state.edit.color_ext.iter_mut() {
        let c_idx = ff.parse_val("reference", n, ColorIdx::default());
        s.set_value(c_idx);
    }

    ctx.show_theme = create_edit_theme(state);

    Ok(())
}

pub fn create_edit_theme(state: &Scenery) -> SalsaTheme {
    let palette = state.edit.palette();
    match state.detail.show.themes.value().as_str() {
        "Shell" => themes::create_shell("Shell", palette),
        "Fallback" => themes::create_fallback("Fallback", palette),
        _ => themes::create_dark("Dark", palette),
    }
}
