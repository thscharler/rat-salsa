use rat_popup::Placement;

pub mod popup_edit;
pub mod popup_focus;
pub mod popup_lock_edit;
pub mod popup_nonfocus;

fn calc_dxy(placement: Placement, d: u16) -> (i16, i16) {
    let neg_d = 0i16.saturating_sub_unsigned(d);
    match placement {
        Placement::None => (0, 0),
        Placement::AboveLeft(_) => (neg_d, 0),
        Placement::AboveCenter(_) => (0, 0),
        Placement::AboveRight(_) => (1, 0),
        Placement::LeftTop(_) => (0, neg_d),
        Placement::LeftMiddle(_) => (0, 0),
        Placement::LeftBottom(_) => (0, 1),
        Placement::RightTop(_) => (0, neg_d),
        Placement::RightMiddle(_) => (0, 0),
        Placement::RightBottom(_) => (0, 1),
        Placement::BelowLeft(_) => (neg_d, 0),
        Placement::BelowCenter(_) => (0, 0),
        Placement::BelowRight(_) => (1, 0),
        Placement::Position(_, _) => (neg_d, neg_d),
        _ => {
            unimplemented!()
        }
    }
}
