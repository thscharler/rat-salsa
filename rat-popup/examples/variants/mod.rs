use rat_popup::PopupConstraint;
use ratatui_core::layout::Alignment;

pub mod popup_edit;
pub mod popup_focus;
pub mod popup_lock_edit;
pub mod popup_nonfocus;

fn calc_dxy(placement: PopupConstraint, d: u16) -> (i16, i16) {
    let neg_d = 0i16.saturating_sub_unsigned(d);
    match placement {
        PopupConstraint::None => (0, 0),
        PopupConstraint::Above(Alignment::Left, _) => (neg_d, 0),
        PopupConstraint::Above(Alignment::Center, _) => (0, 0),
        PopupConstraint::Above(Alignment::Right, _) => (1, 0),
        PopupConstraint::Left(Alignment::Left, _) => (0, neg_d),
        PopupConstraint::Left(Alignment::Center, _) => (0, 0),
        PopupConstraint::Left(Alignment::Right, _) => (0, 1),
        PopupConstraint::Right(Alignment::Left, _) => (0, neg_d),
        PopupConstraint::Right(Alignment::Center, _) => (0, 0),
        PopupConstraint::Right(Alignment::Right, _) => (0, 1),
        PopupConstraint::Below(Alignment::Left, _) => (neg_d, 0),
        PopupConstraint::Below(Alignment::Center, _) => (0, 0),
        PopupConstraint::Below(Alignment::Right, _) => (1, 0),
        PopupConstraint::Position(_, _) => (neg_d, neg_d),
        _ => {
            unimplemented!()
        }
    }
}
