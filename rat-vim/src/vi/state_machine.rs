use crate::coroutine::Yield;
use crate::vi::{History, Mark, Motion, Scrolling, TxtObj, Vim};
use crate::{ctrl, yield_};
use std::cell::RefCell;
use std::rc::Rc;

async fn bare_multiplier(
    mut tok: char,
    motion_buf: &RefCell<String>,
    yp: &Yield<char, Vim>,
) -> (Option<u32>, char) {
    let mut mul = String::new();
    while tok.is_ascii_digit() || tok == ctrl::BS {
        if tok == ctrl::BS {
            mul.pop();
            motion_buf.borrow_mut().pop();
        } else {
            mul.push(tok);
            motion_buf.borrow_mut().push(tok);
        }
        tok = yield_!(yp);
    }
    let mul = mul.parse::<u32>().ok();

    (mul, tok)
}

async fn bare_bare_object(tok: char, mul: Option<u32>, txo: TxtObj) -> Result<Vim, char> {
    match tok {
        'w' => Ok(Vim::Move(mul.unwrap_or(1), Motion::Word(txo))),
        'W' => Ok(Vim::Move(mul.unwrap_or(1), Motion::WORD(txo))),
        's' => Ok(Vim::Move(mul.unwrap_or(1), Motion::Sentence(txo))),
        'p' => Ok(Vim::Move(mul.unwrap_or(1), Motion::Paragraph(txo))),
        't' => Ok(Vim::Move(mul.unwrap_or(1), Motion::Tagged(txo))),
        ']' | '[' => Ok(Vim::Move(mul.unwrap_or(1), Motion::Bracket(txo))),
        ')' | '(' | 'b' => Ok(Vim::Move(mul.unwrap_or(1), Motion::Parenthesis(txo))),
        '}' | '{' | 'B' => Ok(Vim::Move(mul.unwrap_or(1), Motion::Brace(txo))),
        '\'' | '"' | '`' => Ok(Vim::Move(mul.unwrap_or(1), Motion::Quoted(tok, txo))),
        '>' | '<' => Ok(Vim::Move(mul.unwrap_or(1), Motion::Angled(txo))),
        _ => Err(tok),
    }
}

async fn bare_object(
    tok: char,
    mul: Option<u32>,
    motion_buf: &RefCell<String>,
    yp: &Yield<char, Vim>,
) -> Result<Vim, char> {
    match tok {
        'a' => {
            let tok = yield_!(yp);
            motion_buf.borrow_mut().push(tok);
            bare_bare_object(tok, mul, TxtObj::A).await
        }
        'i' => {
            let tok = yield_!(yp);
            motion_buf.borrow_mut().push(tok);
            bare_bare_object(tok, mul, TxtObj::I).await
        }
        _ => Err(tok),
    }
}

async fn bare_motion(
    tok: char,
    mul: Option<u32>,
    motion_buf: &RefCell<String>,
    yp: &Yield<char, Vim>,
) -> Result<Vim, char> {
    match tok {
        'h' => Ok(Vim::Move(mul.unwrap_or(1), Motion::Left)),
        'l' => Ok(Vim::Move(mul.unwrap_or(1), Motion::Right)),
        '-' | 'k' => Ok(Vim::Move(mul.unwrap_or(1), Motion::Up)),
        '+' | 'j' | '\n' => Ok(Vim::Move(mul.unwrap_or(1), Motion::Down)),
        '_' => Ok(Vim::Move(mul.unwrap_or(1).saturating_sub(1), Motion::Down)),
        ctrl::CTRL_U => Ok(Vim::Move(mul.unwrap_or(0), Motion::HalfPageUp)),
        ctrl::CTRL_D => Ok(Vim::Move(mul.unwrap_or(0), Motion::HalfPageDown)),
        'H' => Ok(Vim::Move(mul.unwrap_or(0), Motion::ToTopOfScreen)),
        'M' => Ok(Vim::Move(mul.unwrap_or(0), Motion::ToMiddleOfScreen)),
        'L' => Ok(Vim::Move(mul.unwrap_or(0), Motion::ToBottomOfScreen)),
        '|' => Ok(Vim::Move(mul.unwrap_or(0), Motion::ToCol)),
        'w' => Ok(Vim::Move(mul.unwrap_or(1), Motion::NextWordStart)),
        'b' => Ok(Vim::Move(mul.unwrap_or(1), Motion::PrevWordStart)),
        'e' => Ok(Vim::Move(mul.unwrap_or(1), Motion::NextWordEnd)),
        'g' => {
            let tok = yield_!(yp);
            motion_buf.borrow_mut().push(tok);
            match tok {
                'e' => Ok(Vim::Move(mul.unwrap_or(1), Motion::PrevWordEnd)),
                'E' => Ok(Vim::Move(mul.unwrap_or(1), Motion::PrevWORDEnd)),
                '_' => Ok(Vim::Move(mul.unwrap_or(1), Motion::EndOfLineText)),
                ',' => Ok(Vim::History(mul.unwrap_or(1), History::NextChange)),
                ';' => Ok(Vim::History(mul.unwrap_or(1), History::PrevChange)),
                'g' => {
                    if let Some(mul) = mul {
                        Ok(Vim::Move(mul, Motion::ToLine))
                    } else {
                        Ok(Vim::Move(1, Motion::StartOfFile))
                    }
                }
                _ => Ok(Vim::Invalid),
            }
        }
        'W' => Ok(Vim::Move(mul.unwrap_or(1), Motion::NextWORDStart)),
        'B' => Ok(Vim::Move(mul.unwrap_or(1), Motion::PrevWORDStart)),
        'E' => Ok(Vim::Move(mul.unwrap_or(1), Motion::NextWORDEnd)),
        '(' => Ok(Vim::Move(mul.unwrap_or(1), Motion::PrevSentence)),
        ')' => Ok(Vim::Move(mul.unwrap_or(1), Motion::NextSentence)),
        '{' => Ok(Vim::Move(mul.unwrap_or(1), Motion::PrevParagraph)),
        '}' => Ok(Vim::Move(mul.unwrap_or(1), Motion::NextParagraph)),
        'G' => {
            if let Some(mul) = mul {
                Ok(Vim::Move(mul, Motion::ToLine))
            } else {
                Ok(Vim::Move(1, Motion::EndOfFile))
            }
        }
        '^' => Ok(Vim::Move(1, Motion::StartOfLineText)),
        '$' => Ok(Vim::Move(mul.unwrap_or(1), Motion::EndOfLine)),
        '%' => {
            if let Some(mul) = mul {
                Ok(Vim::Move(mul, Motion::ToLinePercent))
            } else {
                Ok(Vim::Move(1, Motion::ToMatchingBrace))
            }
        }

        'f' => {
            let tok = yield_!(yp);
            motion_buf.borrow_mut().push(tok);
            Ok(Vim::Move(mul.unwrap_or(1), Motion::FindForward(tok)))
        }
        'F' => {
            let tok = yield_!(yp);
            motion_buf.borrow_mut().push(tok);
            Ok(Vim::Move(mul.unwrap_or(1), Motion::FindBack(tok)))
        }
        't' => {
            let tok = yield_!(yp);
            motion_buf.borrow_mut().push(tok);
            Ok(Vim::Move(mul.unwrap_or(1), Motion::FindTillForward(tok)))
        }
        'T' => {
            let tok = yield_!(yp);
            motion_buf.borrow_mut().push(tok);
            Ok(Vim::Move(mul.unwrap_or(1), Motion::FindTillBack(tok)))
        }
        ';' => Ok(Vim::Move(mul.unwrap_or(1), Motion::FindRepeatNext)),
        ',' => Ok(Vim::Move(mul.unwrap_or(1), Motion::FindRepeatPrev)),

        '/' => {
            let mut buf = String::new();
            loop {
                let tok = yield_!(
                    Vim::Partial(mul.unwrap_or(1), Motion::SearchForward(buf.clone()),),
                    yp
                );
                if tok == '\n' {
                    break;
                } else if tok == ctrl::BS {
                    let mut mb = motion_buf.borrow_mut();
                    if mb.len() > 1 {
                        mb.pop();
                    }
                    _ = buf.pop();
                } else {
                    motion_buf.borrow_mut().push(tok);
                    buf.push(tok);
                }
            }
            Ok(Vim::Move(mul.unwrap_or(1), Motion::SearchForward(buf)))
        }
        '?' => {
            let mut buf = String::new();
            loop {
                let tok = yield_!(
                    Vim::Partial(mul.unwrap_or(1), Motion::SearchBack(buf.clone()),),
                    yp
                );
                if tok == '\n' {
                    break;
                } else if tok == '\x08' {
                    let mut mb = motion_buf.borrow_mut();
                    if mb.len() > 1 {
                        mb.pop();
                    }
                    _ = buf.pop();
                } else {
                    motion_buf.borrow_mut().push(tok);
                    buf.push(tok);
                }
            }
            Ok(Vim::Move(mul.unwrap_or(1), Motion::SearchBack(buf)))
        }
        '*' => Ok(Vim::Move(mul.unwrap_or(1), Motion::SearchWordForward)),
        '#' => Ok(Vim::Move(mul.unwrap_or(1), Motion::SearchWordBackward)),
        'n' => Ok(Vim::Move(mul.unwrap_or(1), Motion::SearchRepeatNext)),
        'N' => Ok(Vim::Move(mul.unwrap_or(1), Motion::SearchRepeatPrev)),

        '\'' => {
            let tok = yield_!(yp);
            motion_buf.borrow_mut().push(tok);
            match tok {
                'a'..'z' => Ok(Vim::Move(1, Motion::ToMark(Mark::Char(tok), true))),
                '\'' | '`' => Ok(Vim::Move(1, Motion::ToMark(Mark::Jump, true))),
                '[' => Ok(Vim::Move(1, Motion::ToMark(Mark::ChangeStart, true))),
                ']' => Ok(Vim::Move(1, Motion::ToMark(Mark::ChangeEnd, true))),
                '<' => Ok(Vim::Move(1, Motion::ToMark(Mark::VisualAnchor, true))),
                '>' => Ok(Vim::Move(1, Motion::ToMark(Mark::VisualLead, true))),
                '^' => Ok(Vim::Move(1, Motion::ToMark(Mark::Insert, true))),
                _ => Ok(Vim::Invalid),
            }
        }
        '`' => {
            let tok = yield_!(yp);
            motion_buf.borrow_mut().push(tok);
            match tok {
                'a'..'z' => Ok(Vim::Move(1, Motion::ToMark(Mark::Char(tok), false))),
                '\'' | '`' => Ok(Vim::Move(1, Motion::ToMark(Mark::Jump, false))),
                '[' => Ok(Vim::Move(1, Motion::ToMark(Mark::ChangeStart, false))),
                ']' => Ok(Vim::Move(1, Motion::ToMark(Mark::ChangeEnd, false))),
                '<' => Ok(Vim::Move(1, Motion::ToMark(Mark::VisualAnchor, false))),
                '>' => Ok(Vim::Move(1, Motion::ToMark(Mark::VisualLead, false))),
                '^' => Ok(Vim::Move(1, Motion::ToMark(Mark::Insert, false))),
                _ => Ok(Vim::Invalid),
            }
        }
        ctrl::CTRL_O => Ok(Vim::History(mul.unwrap_or(1), History::PrevJump)),
        ctrl::CTRL_I => Ok(Vim::History(mul.unwrap_or(1), History::NextJump)),

        _ => Err(tok),
    }
}

async fn motion_or_textobject(
    mut tok: char,
    mul: Option<u32>,
    motion_buf: &RefCell<String>,
    yp: &Yield<char, Vim>,
) -> Result<Vim, char> {
    tok = match bare_motion(tok, mul, &motion_buf, &yp).await {
        Ok(v) => return Ok(v),
        Err(tok) => tok,
    };
    tok = match bare_object(tok, mul, &motion_buf, &yp).await {
        Ok(v) => return Ok(v),
        Err(tok) => tok,
    };
    Err(tok)
}

async fn bare_scroll(
    tok: char,
    mul: Option<u32>,
    motion_buf: &RefCell<String>,
    yp: &Yield<char, Vim>,
) -> Result<Vim, char> {
    match tok {
        'z' => {
            let tok = yield_!(yp);
            motion_buf.borrow_mut().push(tok);
            match tok {
                'z' => Ok(Vim::Scroll(1, Scrolling::MiddleOfScreen)),
                't' => Ok(Vim::Scroll(1, Scrolling::TopOfScreen)),
                'b' => Ok(Vim::Scroll(1, Scrolling::BottomOfScreen)),
                _ => Ok(Vim::Invalid),
            }
        }
        ctrl::CTRL_Y => Ok(Vim::Scroll(mul.unwrap_or(1), Scrolling::Up)),
        ctrl::CTRL_E => Ok(Vim::Scroll(mul.unwrap_or(1), Scrolling::Down)),
        ctrl::CTRL_B => Ok(Vim::Scroll(mul.unwrap_or(1), Scrolling::PageUp)),
        ctrl::CTRL_F => Ok(Vim::Scroll(mul.unwrap_or(1), Scrolling::PageDown)),
        _ => Err(tok),
    }
}

pub async fn next_normal(tok: char, motion_buf: Rc<RefCell<String>>, yp: Yield<char, Vim>) -> Vim {
    if tok == '0' {
        return Vim::Move(1, Motion::StartOfLine);
    }

    let (mul, tok) = bare_multiplier(tok, &motion_buf, &yp).await;

    motion_buf.borrow_mut().push(tok);
    let tok = match bare_motion(tok, mul, &motion_buf, &yp).await {
        Ok(v) => return v,
        Err(tok) => tok,
    };
    let tok = match bare_scroll(tok, mul, &motion_buf, &yp).await {
        Ok(v) => return v,
        Err(tok) => tok,
    };

    match tok {
        'm' => {
            let tok = yield_!(yp);
            motion_buf.borrow_mut().push(tok);
            match tok {
                'a'..'z' => Vim::Mark(Mark::Char(tok)),
                '\'' | '`' => Vim::Mark(Mark::Jump),
                '[' => Vim::Mark(Mark::ChangeStart),
                ']' => Vim::Mark(Mark::ChangeEnd),
                '<' => Vim::Mark(Mark::VisualAnchor),
                '>' => Vim::Mark(Mark::VisualLead),
                '^' => Vim::Mark(Mark::Insert),
                _ => Vim::Invalid,
            }
        }

        'v' => Vim::Visual(false),
        ctrl::CTRL_V => Vim::Visual(true),

        'r' => {
            let tok = yield_!(yp);
            motion_buf.borrow_mut().push(tok);
            Vim::Replace(mul.unwrap_or(1), tok)
        }
        'd' => {
            let tok = yield_!(yp);

            let (mul2, tok) = bare_multiplier(tok, &motion_buf, &yp).await;

            motion_buf.borrow_mut().push(tok);
            let tok = match motion_or_textobject(tok, mul2, &motion_buf, &yp).await {
                Ok(Vim::Move(mul2, motion)) => {
                    let mul = mul.unwrap_or(1);
                    return Vim::Delete(mul * mul2, motion);
                }
                Ok(_) => return Vim::Invalid,
                Err(tok) => tok,
            };
            match tok {
                'd' => Vim::Delete(mul.unwrap_or(1), Motion::FullLine),
                _ => Vim::Invalid,
            }
        }
        'c' => {
            let tok = yield_!(yp);

            let (mul2, tok) = bare_multiplier(tok, &motion_buf, &yp).await;

            motion_buf.borrow_mut().push(tok);
            let tok = match motion_or_textobject(tok, mul2, &motion_buf, &yp).await {
                Ok(Vim::Move(mul2, motion)) => {
                    let mul = mul.unwrap_or(1);
                    return Vim::Change(mul * mul2, motion);
                }
                Ok(_) => return Vim::Invalid,
                Err(tok) => tok,
            };
            match tok {
                'c' => Vim::Change(mul.unwrap_or(1), Motion::FullLine),
                _ => Vim::Invalid,
            }
        }
        'y' => {
            let tok = yield_!(yp);

            let (mul2, tok) = bare_multiplier(tok, &motion_buf, &yp).await;

            motion_buf.borrow_mut().push(tok);
            let tok = match motion_or_textobject(tok, mul2, &motion_buf, &yp).await {
                Ok(Vim::Move(mul2, motion)) => {
                    let mul = mul.unwrap_or(1);
                    return Vim::Yank(mul * mul2, motion);
                }
                Ok(_) => return Vim::Invalid,
                Err(tok) => tok,
            };
            match tok {
                'y' => Vim::Yank(mul.unwrap_or(1), Motion::FullLine),
                _ => Vim::Invalid,
            }
        }
        '"' => {
            let tok = yield_!(yp);
            motion_buf.borrow_mut().push(tok);
            match tok {
                '*' => {
                    let tok = yield_!(yp);
                    motion_buf.borrow_mut().push(tok);

                    match tok {
                        'y' => {
                            let tok = yield_!(yp);

                            let (mul2, tok) = bare_multiplier(tok, &motion_buf, &yp).await;

                            motion_buf.borrow_mut().push(tok);
                            let tok = match motion_or_textobject(tok, mul2, &motion_buf, &yp).await
                            {
                                Ok(Vim::Move(mul2, motion)) => {
                                    let mul = mul.unwrap_or(1);
                                    return Vim::CopyClipboard(mul * mul2, motion);
                                }
                                Ok(_) => return Vim::Invalid,
                                Err(tok) => tok,
                            };
                            match tok {
                                'y' => Vim::CopyClipboard(mul.unwrap_or(1), Motion::FullLine),
                                _ => Vim::Invalid,
                            }
                        }
                        'p' => Vim::PasteClipboard(mul.unwrap_or(1), false),
                        'P' => Vim::PasteClipboard(mul.unwrap_or(1), true),
                        _ => Vim::Invalid,
                    }
                }
                // TODO other registers
                _ => Vim::Invalid,
            }
        }
        'p' => Vim::Paste(mul.unwrap_or(1), false),
        'P' => Vim::Paste(mul.unwrap_or(1), true),
        'D' => Vim::Delete(mul.unwrap_or(1), Motion::EndOfLine),
        'C' => Vim::Change(mul.unwrap_or(1), Motion::EndOfLine),
        's' => Vim::Change(mul.unwrap_or(1), Motion::Right),
        'S' => Vim::Change(mul.unwrap_or(1), Motion::EndOfLine),
        'i' => Vim::Insert(mul.unwrap_or(1)),
        'a' => Vim::Append(mul.unwrap_or(1)),
        'o' => Vim::AppendLine(mul.unwrap_or(1)),
        'O' => Vim::PrependLine(mul.unwrap_or(1)),
        'x' => Vim::Delete(mul.unwrap_or(1), Motion::Right),
        'X' => Vim::Delete(mul.unwrap_or(1), Motion::Left),
        'J' => Vim::JoinLines(mul.unwrap_or(1)),
        'u' => Vim::Undo(mul.unwrap_or(1)),
        ctrl::CTRL_R => Vim::Redo(mul.unwrap_or(1)),
        '<' => Vim::Dedent,
        '>' => Vim::Indent,

        '.' => Vim::Repeat(mul.unwrap_or(1)),

        _ => Vim::Invalid,
    }
}

pub async fn next_visual(
    mut tok: char,
    motion_buf: Rc<RefCell<String>>,
    yp: Yield<char, Vim>,
) -> Vim {
    if tok == '0' {
        return Vim::Move(1, Motion::StartOfLine);
    }

    let mul;
    (mul, tok) = bare_multiplier(tok, &motion_buf, &yp).await;

    motion_buf.borrow_mut().push(tok);
    tok = match motion_or_textobject(tok, mul, &motion_buf, &yp).await {
        Ok(v @ Vim::Move(_, _)) => {
            return v;
        }
        Ok(_) => return Vim::Invalid,
        Err(tok) => tok,
    };

    match tok {
        'o' => Vim::VisualSwapLead,
        'O' => Vim::VisualSwapDiagonal,
        'd' => Vim::Delete(1, Motion::Visual),
        'c' => Vim::Change(1, Motion::Visual),
        'y' => Vim::Yank(1, Motion::Visual),
        '"' => {
            tok = yield_!(yp);
            motion_buf.borrow_mut().push(tok);
            match tok {
                '*' => {
                    tok = yield_!(yp);
                    motion_buf.borrow_mut().push(tok);
                    match tok {
                        'y' => Vim::CopyClipboard(1, Motion::Visual),
                        _ => Vim::Invalid,
                    }
                }
                _ => Vim::Invalid,
            }
        }
        _ => Vim::Invalid,
    }
}
