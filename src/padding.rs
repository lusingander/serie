use ratatui::layout::Rect;

pub(crate) const MIN_CONTENT_WIDTH: u16 = 10;
pub(crate) const MIN_CONTENT_HEIGHT: u16 = 3;

/// Clamp a uniform page padding so at least MIN_W x MIN_H remains.
/// If the frame is already smaller than MIN, returns 0.
pub(crate) fn clamp_uniform(requested: u16, width: u16, height: u16) -> u16 {
    if width < MIN_CONTENT_WIDTH || height < MIN_CONTENT_HEIGHT {
        return 0;
    }
    let max_by_width = width.saturating_sub(MIN_CONTENT_WIDTH) / 2;
    let max_by_height = height.saturating_sub(MIN_CONTENT_HEIGHT) / 2;
    requested.min(max_by_width).min(max_by_height)
}

/// Shrink `area` by `pad` on all four sides. If pad is 0 or area is too small, return area.
pub(crate) fn inset_rect(area: Rect, pad: u16) -> Rect {
    if pad == 0 {
        return area;
    }
    let inset = pad.saturating_mul(2);
    if inset >= area.width || inset >= area.height {
        return area;
    }
    Rect::new(
        area.x.saturating_add(pad),
        area.y.saturating_add(pad),
        area.width.saturating_sub(inset),
        area.height.saturating_sub(inset),
    )
}

/// Further reduce uniform padding so `required_graph_width` fits in `term_width`.
/// Never increases padding. Result is 0 if even 0 is required to try (caller still
/// errors based on unpadded terminal).
pub(crate) fn shrink_to_fit_graph(pad: u16, term_width: u16, required_graph_width: u16) -> u16 {
    let max_pad = term_width.saturating_sub(required_graph_width) / 2;
    pad.min(max_pad)
}

#[cfg(test)]
mod tests {
    use ratatui::layout::Rect;
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case(0, 80, 24, 0)]
    #[case(2, 80, 24, 2)]
    #[case(50, 20, 8, 2)]
    #[case(50, 8, 8, 0)]
    fn clamp_uniform_cases(
        #[case] requested: u16,
        #[case] width: u16,
        #[case] height: u16,
        #[case] expected: u16,
    ) {
        assert_eq!(clamp_uniform(requested, width, height), expected);
    }

    #[test]
    fn inset_rect_shrinks_when_it_fits() {
        assert_eq!(
            inset_rect(Rect::new(0, 0, 80, 24), 2),
            Rect::new(2, 2, 76, 20)
        );
    }

    #[test]
    fn inset_rect_keeps_area_when_pad_does_not_fit() {
        let area = Rect::new(0, 0, 5, 5);
        assert_eq!(inset_rect(area, 10), area);
    }

    #[rstest]
    #[case(20, 80, 72, 4)]
    #[case(2, 80, 24, 2)]
    #[case(10, 80, 100, 0)]
    fn shrink_to_fit_graph_cases(
        #[case] pad: u16,
        #[case] term_width: u16,
        #[case] required_graph_width: u16,
        #[case] expected: u16,
    ) {
        assert_eq!(
            shrink_to_fit_graph(pad, term_width, required_graph_width),
            expected
        );
    }
}
