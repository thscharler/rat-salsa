use rat_popup::PopupConstraint;

pub mod popup_edit;
pub mod popup_focus;
pub mod popup_lock_edit;
pub mod popup_nonfocus;

fn calc_dxy(placement: PopupConstraint, d: u16) -> (i16, i16) {
    let neg_d = 0i16.saturating_sub_unsigned(d);
    match placement {
        PopupConstraint::None => (0, 0),
        PopupConstraint::AboveLeft(_) => (neg_d, 0),
        PopupConstraint::AboveCenter(_) => (0, 0),
        PopupConstraint::AboveRight(_) => (1, 0),
        PopupConstraint::LeftTop(_) => (0, neg_d),
        PopupConstraint::LeftMiddle(_) => (0, 0),
        PopupConstraint::LeftBottom(_) => (0, 1),
        PopupConstraint::RightTop(_) => (0, neg_d),
        PopupConstraint::RightMiddle(_) => (0, 0),
        PopupConstraint::RightBottom(_) => (0, 1),
        PopupConstraint::BelowLeft(_) => (neg_d, 0),
        PopupConstraint::BelowCenter(_) => (0, 0),
        PopupConstraint::BelowRight(_) => (1, 0),
        PopupConstraint::Position(_, _) => (neg_d, neg_d),
        _ => {
            unimplemented!()
        }
    }
}
