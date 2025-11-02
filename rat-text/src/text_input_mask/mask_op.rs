use crate::core::{TextCore, TextString};
use crate::text_input_mask::MaskedInputState;
use crate::text_input_mask::mask_token::{EditDirection, Mask, MaskToken};
use crate::{TextError, TextPosition, TextRange, upos_type};
use format_num_pattern::core::{clean_num, map_num};
use format_num_pattern::{CurrencySym, NumberFormat, NumberSymbols};
use std::ops::Range;
use unicode_segmentation::UnicodeSegmentation;

/// Remove the selection
pub fn remove_range(
    state: &mut MaskedInputState,
    range: Range<upos_type>,
) -> Result<bool, TextError> {
    // check valid range
    state
        .value
        .bytes_at_range(TextRange::new((range.start, 0), (range.end, 0)))?;

    if range.is_empty() {
        return Ok(false);
    }

    let mask = &state.mask[range.start as usize];
    if range.start >= mask.sub_start && range.end <= mask.sub_end {
        if mask.right.is_rtol() {
            state.value.begin_undo_seq();
            state
                .value
                .remove_str_range(TextRange::new((range.start, 0), (range.end, 0)))
                .expect("valid_range");
            let fill_before =
                &state.mask[mask.sub_start as usize..mask.sub_start as usize + range.len()];
            state
                .value
                .insert_str(
                    TextPosition::new(mask.sub_start, 0),
                    &MaskToken::empty_section(fill_before),
                )
                .expect("valid_range");
            reformat(&mut state.value, &state.mask, mask.sub_start..mask.sub_end);
            state.value.end_undo_seq();
        } else if mask.right.is_ltor() {
            state.value.begin_undo_seq();
            state
                .value
                .remove_str_range(TextRange::new((range.start, 0), (range.end, 0)))
                .expect("valid_range");
            let fill_after =
                &state.mask[mask.sub_end as usize - range.len()..mask.sub_end as usize];
            state
                .value
                .insert_str(
                    TextPosition::new(mask.sub_end - range.len() as upos_type, 0),
                    &MaskToken::empty_section(fill_after),
                )
                .expect("valid_range");
            reformat(&mut state.value, &state.mask, mask.sub_start..mask.sub_end);
            state.value.end_undo_seq();
        }

        return Ok(true);
    }

    let mut pos = range.start;
    state.value.begin_undo_seq();
    loop {
        let mask = &state.mask[pos as usize];

        if mask.sub_start < range.start {
            // partial start
            if mask.right.is_rtol() {
                state
                    .value
                    .remove_str_range(TextRange::new((range.start, 0), (mask.sub_end, 0)))
                    .expect("valid_range");

                let len = mask.sub_end - range.start;
                let fill_before =
                    &state.mask[mask.sub_start as usize..(mask.sub_start + len) as usize];
                state
                    .value
                    .insert_str(
                        TextPosition::new(mask.sub_start, 0),
                        &MaskToken::empty_section(fill_before),
                    )
                    .expect("valid_range");

                reformat(&mut state.value, &state.mask, mask.sub_start..mask.sub_end);

                pos = mask.sub_end;
            } else if mask.right.is_ltor() {
                state
                    .value
                    .remove_str_range(TextRange::new((range.start, 0), (mask.sub_end, 0)))
                    .expect("valid_range");

                let fill_after = &state.mask[range.start as usize..mask.sub_end as usize];
                state
                    .value
                    .insert_str(
                        TextPosition::new(range.start, 0),
                        &MaskToken::empty_section(fill_after),
                    )
                    .expect("valid_range");

                reformat(&mut state.value, &state.mask, mask.sub_start..mask.sub_end);

                pos = mask.sub_end;
            }
        } else if mask.sub_end > range.end {
            // partial end
            if mask.right.is_rtol() {
                state
                    .value
                    .remove_str_range(TextRange::new((mask.sub_start, 0), (range.end, 0)))
                    .expect("valid_range");

                let fill_before = &state.mask[mask.sub_start as usize..range.end as usize];
                state
                    .value
                    .insert_str(
                        TextPosition::new(mask.sub_start, 0),
                        &MaskToken::empty_section(fill_before),
                    )
                    .expect("valid_range");

                reformat(&mut state.value, &state.mask, mask.sub_start..mask.sub_end);
                pos = mask.sub_end;
            } else if mask.right.is_ltor() {
                state
                    .value
                    .remove_str_range(TextRange::new((mask.sub_start, 0), (range.end, 0)))
                    .expect("valid_range");

                let len = range.end - mask.sub_start;
                let fill_after = &state.mask[(mask.sub_end - len) as usize..mask.sub_end as usize];
                state
                    .value
                    .insert_str(
                        TextPosition::new(mask.sub_end - len, 0),
                        &MaskToken::empty_section(fill_after),
                    )
                    .expect("valid_range");

                pos = mask.sub_end;
            }
        } else {
            // full section
            state
                .value
                .remove_str_range(TextRange::new((mask.sub_start, 0), (mask.sub_end, 0)))
                .expect("valid_range");

            let sec_range = &state.mask[mask.sub_start as usize..mask.sub_end as usize];
            state
                .value
                .insert_str(
                    TextPosition::new(mask.sub_start, 0),
                    &MaskToken::empty_section(sec_range),
                )
                .expect("valid_range");

            pos = mask.sub_end;
        }

        if pos >= range.end {
            break;
        }
    }
    state.value.end_undo_seq();

    Ok(true)
}

/// Rebuild a section according to number-formatting.
/// The main purpose is to rebuild the grouping separators.
fn reformat(core: &mut TextCore<TextString>, mask: &[MaskToken], section: Range<upos_type>) {
    if mask[section.start as usize].right.is_rtol() {
        let cursor = core.cursor();
        let anchor = core.anchor();

        let sec_str = core
            .str_slice(TextRange::new((section.start, 0), (section.end, 0)))
            .expect("valid_range");

        // to be safe, always use our internal symbol set.
        let sym = NumberSymbols {
            decimal_sep: '.',
            decimal_grp: Some(','),
            negative_sym: '-',
            positive_sym: ' ',
            exponent_upper_sym: 'E',
            exponent_lower_sym: 'e',
            currency_sym: CurrencySym::new("$"),
        };

        // remove all non numbers and leading 0.
        let mut clean = String::new();
        _ = clean_num(sec_str.as_ref(), &sym, &mut clean);

        // create number format
        let mut tok = String::new();
        let submask = &mask[section.start as usize..section.end as usize];
        // default fmt.sym is nice
        for t in submask {
            match &t.right {
                Mask::Digit0(_) => tok.push('0'),
                Mask::Digit(_) => tok.push('9'),
                Mask::Numeric(_) => tok.push('#'),
                Mask::DecimalSep => tok.push('.'),
                Mask::GroupingSep => tok.push(','),
                Mask::Sign => tok.push('-'),
                Mask::Plus => tok.push('+'),
                Mask::Separator(s) => {
                    for c in s.chars() {
                        tok.push('\\');
                        tok.push(c);
                    }
                }
                Mask::None => {}
                _ => unreachable!("invalid mask"),
            }
        }

        let fmt = match NumberFormat::news(tok, sym) {
            Ok(v) => v,
            Err(_) => unreachable!("invalid mask"),
        };
        let mut out = String::new();
        match map_num::<_, false>(clean.as_str(), &fmt, fmt.sym(), &mut out) {
            Ok(_) => {}
            Err(_) => unreachable!("invalid mask"),
        }

        core.remove_char_range(TextRange::new((section.start, 0), (section.end, 0)))
            .expect("valid_range");
        core.insert_str(TextPosition::new(section.start, 0), &out)
            .expect("valid_position");

        // keep cursor intact
        core.set_cursor(anchor, false);
        core.set_cursor(cursor, true);
    } else if mask[section.start as usize].right.is_ltor() {
        let cursor = core.cursor();
        let anchor = core.anchor();

        let sec_str = core
            .str_slice(TextRange::new((section.start, 0), (section.end, 0)))
            .expect("valid_range");
        let sec_mask = &mask[section.start as usize..section.end as usize];
        let mut str_new = String::new();
        for (g, t) in sec_str.graphemes(true).zip(sec_mask.iter()) {
            match t.right {
                Mask::Digit0(_) | Mask::Hex0 | Mask::Oct0 | Mask::Dec0 => {
                    if g == " " {
                        str_new.push('0');
                    } else {
                        str_new.push_str(g);
                    }
                }
                _ => {
                    str_new.push_str(g);
                }
            }
        }

        if sec_str != str_new {
            core.remove_char_range(TextRange::new((section.start, 0), (section.end, 0)))
                .expect("valid_range");
            core.insert_str(TextPosition::new(section.start, 0), &str_new)
                .expect("valid_position");

            // keep cursor intact
            core.set_cursor(anchor, false);
            core.set_cursor(cursor, true);
        }
    }
}

/// Start at the cursor position and find a valid insert position for the input c.
/// Put the cursor at that position.
#[allow(clippy::if_same_then_else)]
pub fn advance_cursor(state: &mut MaskedInputState, c: char) -> bool {
    if state.mask.is_empty() {
        return false;
    }

    let mask_c = &state.mask[state.value.cursor().x as usize];

    let mut new_cursor = state.value.cursor().x;

    loop {
        let mask = &state.mask[new_cursor as usize];

        if can_insert_integer_left(state, mask, new_cursor, c) {
            // At the gap between an integer field and something else.
            // Integer fields are served first.
            break;
        } else if can_insert_integer(state, mask, new_cursor, c) {
            // Insert position inside an integer field. After any spaces
            // and the sign.
            break;
        } else if can_insert_sign(state, mask, new_cursor, c) {
            // Can insert a sign here.
            break;
        } else if can_insert_decimal_sep(state, mask, c) {
            // Decimal separator matches.
            break;
        } else if mask.right == Mask::GroupingSep {
            // Never stop here.
            new_cursor += 1;
        } else if can_insert_separator(state, mask, c) {
            break;
        } else if can_move_left_in_fraction(state, mask_c, mask, new_cursor, c) {
            // skip left
            new_cursor -= 1;
        } else if can_insert_fraction(state, mask_c, mask, c) {
            break;
        } else if can_insert_other(state, mask, c) {
            break;
        } else if mask.right == Mask::None {
            // No better position found. Reset and break;
            new_cursor = state.value.cursor().x;
            break;
        } else {
            new_cursor += 1;
        }
    }

    state
        .value
        .set_cursor(TextPosition::new(new_cursor, 0), false)
}

// Can edit the field left of the cursor.
#[inline]
fn can_insert_integer_left(
    state: &MaskedInputState,
    mask: &MaskToken,
    new_cursor: upos_type,
    c: char,
) -> bool {
    if !mask.peek_left.is_rtol() {
        return false;
    }
    if !mask.right.is_ltor() && !mask.right.is_none() {
        return false;
    }

    let left = &state.mask[new_cursor as usize - 1];
    if !is_valid_char(state, &left.right, c) {
        return false;
    }

    let mask0 = &state.mask[left.sub_start as usize];
    let g0 = state
        .value
        .grapheme_at(TextPosition::new(left.sub_start, 0))
        .expect("valid_position")
        .expect("grapheme");
    if !mask0.right.can_drop(g0.grapheme()) {
        return false;
    }

    true
}

// Is this the correct input position for a rtol field
#[inline]
fn can_insert_integer(
    state: &MaskedInputState,
    mask: &MaskToken,
    new_cursor: upos_type,
    c: char,
) -> bool {
    if !mask.right.is_rtol() {
        return false;
    }

    if !is_valid_char(state, &mask.right, c) {
        return false;
    }

    let g = state
        .value
        .grapheme_at(TextPosition::new(new_cursor, 0))
        .expect("valid_position")
        .expect("grapheme");
    if mask.right.can_drop(g.grapheme()) {
        return false;
    }
    if g == "-" {
        return false;
    }

    true
}

// Can input a sign here?
#[inline]
fn can_insert_sign<'a>(
    state: &'a MaskedInputState,
    mut mask: &'a MaskToken,
    new_cursor: upos_type,
    c: char,
) -> bool {
    if !is_valid_char(state, &Mask::Sign, c) {
        return false;
    }
    // boundary right/left. prefer right, change mask.
    if mask.peek_left.is_number() && (mask.right.is_ltor() || mask.right.is_none()) {
        mask = &state.mask[new_cursor as usize - 1];
    }
    if !mask.right.is_number() {
        return false;
    }

    // check possible positions for the sign.
    for i in mask.sec_start..mask.sec_end {
        let t = &state.mask[i as usize];
        match t.right {
            Mask::Plus => return true,
            Mask::Sign => return true,
            Mask::Numeric(EditDirection::Rtol) => {
                // Numeric fields can hold a sign.
                // If they are not otherwise occupied.
                let gi = state
                    .value
                    .grapheme_at(TextPosition::new(i, 0))
                    .expect("valid_position")
                    .expect("grapheme");

                return t.right.can_drop(gi.grapheme()) || gi == "-";
            }
            _ => {}
        }
    }

    false
}

// Can insert a decimal separator.
#[inline]
fn can_insert_decimal_sep(state: &MaskedInputState, mask: &MaskToken, c: char) -> bool {
    if mask.right != Mask::DecimalSep {
        return false;
    }
    if !is_valid_char(state, &mask.right, c) {
        return false;
    }
    true
}

// Separator char matches
#[inline]
fn can_insert_separator(state: &MaskedInputState, mask: &MaskToken, c: char) -> bool {
    if !matches!(mask.right, Mask::Separator(_)) {
        return false;
    }
    if !is_valid_char(state, &mask.right, c) {
        return false;
    }
    true
}

// When inserting to the fraction we want to left-align
// the digits. This checks if a digit could possibly be
// inserted to the left of the current position.
#[inline]
fn can_move_left_in_fraction(
    state: &MaskedInputState,
    mask_c: &MaskToken,
    mask: &MaskToken,
    new_cursor: upos_type,
    c: char,
) -> bool {
    if !mask.peek_left.is_fraction() {
        return false;
    }
    if !is_valid_char(state, &mask.peek_left, c) {
        return false;
    }
    // don't jump from integer to fraction
    if mask_c.is_integer_part() {
        return false;
    }

    let gl = state
        .value
        .grapheme_at(TextPosition::new(new_cursor - 1, 0))
        .expect("valid_position")
        .expect("grapheme");

    // is there space to the left?
    if gl != " " {
        return false;
    }

    true
}

// Can insert fraction.
#[inline]
fn can_insert_fraction(
    state: &MaskedInputState,
    mask_c: &MaskToken,
    mask: &MaskToken,
    c: char,
) -> bool {
    if !mask.right.is_fraction() {
        return false;
    }
    if !is_valid_char(state, &mask.right, c) {
        return false;
    }
    // don't jump from integer to fraction
    if mask_c.is_integer_part() {
        return false;
    }

    true
}

// Can insert other field types
#[inline]
fn can_insert_other(state: &MaskedInputState, mask: &MaskToken, c: char) -> bool {
    match mask.right {
        Mask::Hex0
        | Mask::Hex
        | Mask::Oct0
        | Mask::Oct
        | Mask::Dec0
        | Mask::Dec
        | Mask::Letter
        | Mask::LetterOrDigit
        | Mask::LetterDigitSpace
        | Mask::AnyChar => is_valid_char(state, &mask.right, c),
        _ => false,
    }
}

/// Valid input for this mask.
fn is_valid_char(state: &MaskedInputState, mask: &Mask, c: char) -> bool {
    match mask {
        Mask::Digit0(_) => c.is_ascii_digit(),
        Mask::Digit(_) => c.is_ascii_digit() || c == ' ',
        Mask::Numeric(_) => c.is_ascii_digit() || c == state.neg_sym() || c == '-',
        Mask::DecimalSep => c == state.dec_sep(),
        Mask::GroupingSep => false,
        Mask::Sign => c == state.neg_sym() || c == '-',
        Mask::Plus => c == state.neg_sym() || c == '-',
        Mask::Hex0 => c.is_ascii_hexdigit(),
        Mask::Hex => c.is_ascii_hexdigit() || c == ' ',
        Mask::Oct0 => c.is_digit(8),
        Mask::Oct => c.is_digit(8) || c == ' ',
        Mask::Dec0 => c.is_ascii_digit(),
        Mask::Dec => c.is_ascii_digit() || c == ' ',
        Mask::Letter => c.is_alphabetic(),
        Mask::LetterOrDigit => c.is_alphanumeric(),
        Mask::LetterDigitSpace => c.is_alphanumeric() || c == ' ',
        Mask::AnyChar => true,
        Mask::Separator(sep) => {
            // ',' and '.' match any separator.
            if c == '.' || c == ',' {
                true
            } else if let Some(sepc) = sep.chars().next() {
                sepc == c
            } else {
                false
            }
        }
        Mask::None => false,
    }
}

/// Insert the char if it matches the cursor mask and the current section is not full.
///
/// `advance_cursor()` must be called before for correct functionality.
///
/// Otherwise: your mileage might vary.
pub fn insert_char(state: &mut MaskedInputState, c: char) -> bool {
    if state.mask.is_empty() {
        return false;
    }

    let cursor = state.value.cursor();

    // note: because of borrow checker. calls &mut methods.
    {
        let mask = &state.mask[cursor.x as usize];
        if mask.right.is_number() && can_insert_sign(state, mask, cursor.x, c) {
            if insert_sign(state, c) {
                return true;
            }
        }
    }
    {
        let mask = &state.mask[cursor.x as usize];
        if mask.peek_left.is_number() && (mask.right.is_ltor() || mask.right.is_none()) {
            let left = &state.mask[cursor.x as usize - 1];
            if can_insert_sign(state, left, cursor.x, c) {
                if insert_sign(state, c) {
                    return true;
                }
            }
        }
    }
    {
        let mask = &state.mask[cursor.x as usize];
        if mask.right.is_rtol() {
            if insert_rtol(state, c) {
                return true;
            }
        }
    }
    {
        let mask = &state.mask[cursor.x as usize];
        if mask.peek_left.is_rtol() && (mask.right.is_ltor() || mask.right.is_none()) {
            if insert_rtol(state, c) {
                return true;
            }
        }
    }
    {
        let mask = &state.mask[cursor.x as usize];
        if mask.right.is_ltor() {
            if insert_ltor(state, c) {
                #[allow(clippy::needless_return)]
                return true;
            }
        }
    }

    false
}

/// Insert a sign c into the current number section
#[allow(clippy::single_match)]
fn insert_sign(state: &mut MaskedInputState, c: char) -> bool {
    let cursor = state.value.cursor();

    let mut mask = &state.mask[cursor.x as usize];
    // boundary right/left. prefer right, change mask.
    if mask.peek_left.is_number() && (mask.right.is_ltor() || mask.right.is_none()) {
        mask = &state.mask[cursor.x as usize - 1];
    }

    // explicit sign?
    let idx = state.mask[mask.sec_start as usize..mask.sec_end as usize]
        .iter()
        .enumerate()
        .find(|(_, t)| matches!(t.right, Mask::Sign | Mask::Plus))
        .map(|(i, _)| mask.sec_start as usize + i);

    // existing sign somewhere?
    let idx = if idx.is_none() {
        state
            .value
            .graphemes(
                TextRange::new((mask.sec_start, 0), (mask.sec_end, 0)),
                TextPosition::new(mask.sec_start, 0),
            )
            .expect("valid_range")
            .enumerate()
            .find(|(_, g)| *g == "-" || *g == "+")
            .map(|(i, _)| mask.sec_start as usize + i)
    } else {
        idx
    };

    let idx = if idx.is_none() {
        // moving sign
        let mut idx = mask.sec_end - 1;
        'f: {
            while idx >= mask.sec_start {
                if state.mask[idx as usize].right == Mask::Numeric(EditDirection::Rtol) {
                    let g = state
                        .grapheme_at(idx)
                        .expect("valid_position")
                        .expect("grapheme");

                    if state.mask[idx as usize].right.can_drop(g.grapheme()) {
                        break 'f Some(idx as usize);
                    }
                }
                idx -= 1;
            }
            None
        }
    } else {
        idx
    };

    if let Some(idx) = idx {
        let mask_sign = &state.mask[idx];

        if c == state.neg_sym() || c == '-' {
            // negate current
            let g = state
                .value
                .str_slice(TextRange::new(
                    (idx as upos_type, 0),
                    (idx as upos_type + 1, 0),
                ))
                .expect("valid_pos")
                .to_string();

            state.value.begin_undo_seq();
            state
                .value
                .remove_char_range(TextRange::new(
                    (idx as upos_type, 0),
                    (idx as upos_type + 1, 0),
                ))
                .expect("valid_range");

            let cc = match &mask_sign.right {
                Mask::Numeric(_) | Mask::Sign => {
                    if g == "-" {
                        ' '
                    } else {
                        '-'
                    }
                }
                Mask::Plus => {
                    if g == "-" {
                        '+'
                    } else {
                        '-'
                    }
                }
                _ => unreachable!(),
            };

            state
                .value
                .insert_char(TextPosition::new(idx as upos_type, 0), cc)
                .expect("valid_range");
            state.set_cursor(cursor.x, false);
            state.value.end_undo_seq();
            true
        } else {
            false
        }
    } else {
        false
    }
}

/// Insert c into a rtol section
fn insert_rtol(state: &mut MaskedInputState, c: char) -> bool {
    let cursor = state.value.cursor();

    let mut mask = &state.mask[cursor.x as usize];

    // boundary right/left. prefer right, change mask.
    if mask.peek_left.is_rtol() && (mask.right.is_ltor() || mask.right.is_none()) {
        mask = &state.mask[cursor.x as usize - 1];
    }

    let mask0 = &state.mask[mask.sub_start as usize];

    let g0 = state
        .value
        .grapheme_at(TextPosition::new(mask.sub_start, 0))
        .expect("valid_pos")
        .expect("grapheme");
    if mask0.right.can_drop(g0.grapheme()) && is_valid_char(state, &mask.right, c) {
        state.value.begin_undo_seq();
        state
            .value
            .remove_char_range(TextRange::new((mask.sub_start, 0), (mask.sub_start + 1, 0)))
            .expect("valid_position");
        state
            .value
            .insert_char(TextPosition::new(cursor.x - 1, 0), c)
            .expect("valid_position");
        reformat(&mut state.value, &state.mask, mask.sub_start..mask.sub_end);
        state.value.end_undo_seq();
        return true;
    }

    false
}

/// Insert c into a ltor section.
fn insert_ltor(state: &mut MaskedInputState, c: char) -> bool {
    let cursor = state.value.cursor();

    let mask = &state.mask[cursor.x as usize];
    let mask9 = &state.mask[mask.sub_end as usize - 1];

    // overwrite digit in fraction?
    let g = state
        .value
        .grapheme_at(cursor)
        .expect("valid_cursor")
        .expect("mask");
    if mask.right.is_fraction()
        && mask.right.can_overwrite_fraction(g.grapheme())
        && is_valid_char(state, &mask.right, c)
    {
        // to the right only defaults
        let frac_mask = &state.mask[cursor.x as usize + 1..mask.sub_end as usize];
        let frac_str = state
            .value
            .str_slice(TextRange::new((cursor.x + 1, 0), (mask.sub_end, 0)))
            .expect("valid_range");
        if frac_str == MaskToken::empty_section(frac_mask) {
            state.value.begin_undo_seq();
            state
                .value
                .remove_char_range(TextRange::new(cursor, (cursor.x + 1, 0)))
                .expect("valid_cursor");
            state.value.insert_char(cursor, c).expect("valid_cursor");
            state.value.end_undo_seq();
            return true;
        }
    }

    let g = state
        .value
        .grapheme_at(cursor)
        .expect("valid_cursor")
        .expect("mask");
    if mask.right.can_overwrite(g.grapheme()) && is_valid_char(state, &mask.right, c) {
        if mask.right.is_separator() {
            state.value.begin_undo_seq();
            let r = if let Some(next) = state.next_section_cursor(cursor.x) {
                state.value.set_cursor(TextPosition::new(next, 0), false)
            } else {
                state
                    .value
                    .set_cursor(TextPosition::new(state.line_width(), 0), false)
            };
            state.value.end_undo_seq();
            return r;
        } else if mask.right == Mask::DecimalSep {
            state.value.begin_undo_seq();
            state
                .value
                .set_cursor(TextPosition::new(cursor.x + 1, 0), false);
            state.value.end_undo_seq();
            return true;
        } else {
            state.value.begin_undo_seq();
            state
                .value
                .remove_char_range(TextRange::new(cursor, (cursor.x + 1, 0)))
                .expect("valid_cursor");
            state.value.insert_char(cursor, c).expect("valid_cursor");
            state.value.end_undo_seq();
            return true;
        }
    }

    // can shift right
    let g9 = state
        .value
        .grapheme_at(TextPosition::new(mask.sub_end - 1, 0))
        .expect("valid_pos")
        .expect("mask");
    if mask9.right.can_drop(g9.grapheme()) && is_valid_char(state, &mask.right, c) {
        state.value.begin_undo_seq();
        state
            .value
            .remove_char_range(TextRange::new((mask.sub_end - 1, 0), (mask.sub_end, 0)))
            .expect("valid_range");
        state.value.insert_char(cursor, c).expect("valid_cursor");
        state.value.end_undo_seq();
        return true;
    }
    false
}

/// Remove the previous char.
pub fn remove_next(state: &mut MaskedInputState) {
    let cursor = state.value.cursor();

    if cursor.x as usize == state.mask.len() - 1 {
        return;
    }

    let right = &state.mask[cursor.x as usize];

    // remove and fill with empty
    if right.right.is_rtol() {
        let l0 = &state.mask[right.sub_start as usize];

        state.value.begin_undo_seq();
        state
            .value
            .remove_char_range(TextRange::new(cursor, (cursor.x + 1, 0)))
            .expect("valid_range");
        state
            .value
            .insert_str(TextPosition::new(right.sub_start, 0), &l0.edit)
            .expect("valid_position");
        reformat(
            &mut state.value,
            &state.mask,
            right.sub_start..right.sub_end,
        );

        state
            .value
            .set_cursor(TextPosition::new(cursor.x + 1, 0), false);

        state.value.end_undo_seq();
    } else if right.right.is_ltor() {
        // Check if the section is empty
        let sec_str = state
            .value
            .str_slice(TextRange::new((right.sub_start, 0), (right.sub_end, 0)))
            .expect("valid_range");
        let sec_mask = &state.mask[right.sub_start as usize..right.sub_end as usize];
        let sec_empty = sec_str == MaskToken::empty_section(sec_mask);

        let l9 = &state.mask[right.sub_end as usize - 1];

        state.value.begin_undo_seq();
        state
            .value
            .remove_char_range(TextRange::new(cursor, (cursor.x + 1, 0)))
            .expect("valid_range");
        state
            .value
            .insert_str(TextPosition::new(right.sub_end - 1, 0), &l9.edit)
            .expect("valid_position");

        reformat(
            &mut state.value,
            &state.mask,
            right.sub_start..right.sub_end,
        );

        // in a ltor field keep the cursor at the same position until the
        // whole section is empty. Only then put it at the end of the section
        // to continue right of the section.
        if sec_empty {
            state
                .value
                .set_cursor(TextPosition::new(right.sub_end, 0), false);
        } else {
            // cursor stays
        }

        state.value.end_undo_seq();
    }
}

/// Remove the previous char.
pub fn remove_prev(state: &mut MaskedInputState) {
    let cursor = state.value.cursor();

    if cursor.x == 0 {
        return;
    }

    let left = &state.mask[cursor.x as usize - 1];

    if left.right.is_rtol() {
        // Check if the section is empty
        let sec_empty = if left.right.is_rtol() {
            let sec_str = state
                .value
                .str_slice(TextRange::new((left.sub_start, 0), (left.sub_end, 0)))
                .expect("valid_range");
            let sec_mask = &state.mask[left.sub_start as usize..left.sub_end as usize];
            sec_str == MaskToken::empty_section(sec_mask)
        } else {
            false
        };

        let l0 = &state.mask[left.sub_start as usize];

        state.value.begin_undo_seq();
        state
            .value
            .remove_char_range(TextRange::new((cursor.x - 1, 0), cursor))
            .expect("valid_range");
        state
            .value
            .insert_str(TextPosition::new(left.sub_start, 0), &l0.edit)
            .expect("valid_position");
        reformat(&mut state.value, &state.mask, left.sub_start..left.sub_end);

        // in a rtol field keep the cursor at the same position until the
        // whole section is empty. Only then put it at the beginning of the section
        // to continue left of the section.
        if sec_empty {
            state
                .value
                .set_cursor(TextPosition::new(left.sub_start, 0), false);
        } else {
            // cursor stays
        }

        state.value.end_undo_seq();
    } else if left.right.is_ltor() {
        let l9 = &state.mask[left.sub_end as usize - 1];

        state.value.begin_undo_seq();
        state
            .value
            .remove_char_range(TextRange::new((cursor.x - 1, 0), cursor))
            .expect("valid_range");
        state
            .value
            .insert_str(TextPosition::new(left.sub_end - 1, 0), &l9.edit)
            .expect("valid_position");

        reformat(&mut state.value, &state.mask, left.sub_start..left.sub_end);

        state
            .value
            .set_cursor(TextPosition::new(cursor.x - 1, 0), false);

        state.value.end_undo_seq();
    }
}
