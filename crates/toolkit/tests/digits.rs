use toolkit::digits::{format_value, layout, segments, Align, Bars, Style, A, B, C, D, E, F, G};
use toolkit::geometry::Rect;

const SCALES: [f32; 6] = [1.0, 1.25, 1.5, 1.75, 2.0, 2.5];
const HEIGHTS: [f32; 3] = [14.0, 20.0, 40.0];

fn count(c: char) -> u32 {
    segments(c).count_ones()
}

fn bars(text: &str, scale: f32, style: Style) -> Bars {
    layout(
        text,
        Rect::new(0.0, 0.0, 400.0, 100.0),
        scale,
        Align::Start,
        style,
    )
}

fn all(b: &Bars) -> Vec<Rect> {
    b.lit.iter().chain(b.unlit.iter()).copied().collect()
}

fn overlap(a: Rect, b: Rect) -> bool {
    a.x < b.x + b.w && b.x < a.x + a.w && a.y < b.y + b.h && b.y < a.y + a.h
}

#[test]
fn the_digits_light_the_bars_a_calculator_does() {
    assert_eq!(segments('8'), A | B | C | D | E | F | G);
    assert_eq!(segments('1'), B | C);
    assert_eq!(segments('7'), A | B | C);
    assert_eq!(segments('0') & G, 0);
    assert_eq!(segments('-'), G);
    let counts: Vec<u32> = "0123456789".chars().map(count).collect();
    assert_eq!(counts, [6, 2, 5, 5, 4, 5, 6, 3, 7, 6]);
}

#[test]
fn blanks_the_dot_and_unknown_characters_light_no_bar() {
    for c in [' ', '.', 'x', '+', 'é'] {
        assert_eq!(segments(c), 0, "{c:?}");
    }
}

#[test]
fn values_are_written_with_the_asked_places() {
    assert_eq!(format_value(12.5, 1), "12.5");
    assert_eq!(format_value(12.5, 0), "12");
    assert_eq!(format_value(3.14159, 2), "3.14");
    assert_eq!(format_value(-7.0, 1), "-7.0");
    assert_eq!(format_value(100.0, 0), "100");
}

#[test]
fn a_small_negative_that_rounds_to_zero_has_no_minus() {
    assert_eq!(format_value(-0.04, 1), "0.0");
    assert_eq!(format_value(-0.0, 0), "0");
    assert_eq!(format_value(-0.5, 1), "-0.5");
}

#[test]
fn something_that_is_not_a_number_shows_dashes() {
    for v in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        assert_eq!(format_value(v, 1), "---");
    }
}

#[test]
fn each_cell_draws_its_lit_bars_and_the_rest_dim() {
    let style = Style::new(20.0);
    let eight = bars("8", 1.0, style);
    assert_eq!((eight.lit.len(), eight.unlit.len()), (7, 0));
    let one = bars("1", 1.0, style);
    assert_eq!((one.lit.len(), one.unlit.len()), (2, 5));
    let minus = bars("-", 1.0, style);
    assert_eq!((minus.lit.len(), minus.unlit.len()), (1, 6));
    let blank = bars(" ", 1.0, style);
    assert_eq!((blank.lit.len(), blank.unlit.len()), (0, 7));
    let dot = bars(".", 1.0, style);
    assert_eq!((dot.lit.len(), dot.unlit.len()), (1, 0));
}

#[test]
fn a_number_is_the_sum_of_its_cells() {
    let b = bars("12.5", 1.0, Style::new(20.0));
    // 1 -> 2 lit, 2 -> 5, dot -> 1, 5 -> 5.
    assert_eq!(b.lit.len(), 13);
    assert_eq!(b.unlit.len(), 5 + 2 + 2);
}

#[test]
fn nothing_for_empty_text() {
    let b = bars("", 1.0, Style::new(20.0));
    assert!(b.lit.is_empty() && b.unlit.is_empty());
    assert_eq!(Style::new(20.0).width("", 1.0), 0.0);
}

#[test]
fn at_every_common_scale_all_bars_sit_on_whole_pixels_and_are_visible() {
    for height in HEIGHTS {
        for scale in SCALES {
            for r in all(&bars("-0123456789. ", scale, Style::new(height))) {
                assert!(
                    [r.x, r.y, r.w, r.h].iter().all(|v| *v == v.round()),
                    "{height} at {scale}: {r:?}"
                );
                assert!(r.w >= 1.0 && r.h >= 1.0, "{height} at {scale}: {r:?}");
            }
        }
    }
}

#[test]
fn the_bars_of_a_digit_never_overlap() {
    for height in HEIGHTS {
        for scale in SCALES {
            let rects = all(&bars("8", scale, Style::new(height)));
            for (i, a) in rects.iter().enumerate() {
                for b in &rects[i + 1..] {
                    assert!(!overlap(*a, *b), "{height} at {scale}: {a:?} {b:?}");
                }
            }
        }
    }
}

#[test]
fn neighbouring_cells_never_overlap() {
    for scale in SCALES {
        let rects = all(&bars("88.8", scale, Style::new(20.0)));
        for (i, a) in rects.iter().enumerate() {
            for b in &rects[i + 1..] {
                assert!(!overlap(*a, *b), "at {scale}: {a:?} {b:?}");
            }
        }
    }
}

#[test]
fn cells_run_from_left_to_right() {
    let style = Style::new(20.0);
    let one = all(&bars("8", 1.0, style));
    let two = all(&bars("88", 1.0, style));
    let right_of_first = one.iter().map(|r| r.x + r.w).fold(0.0, f32::max);
    let second_cell = &two[one.len()..];
    assert!(second_cell.iter().all(|r| r.x > right_of_first));
}

#[test]
fn the_width_is_what_the_layout_uses() {
    let style = Style::new(20.0);
    for scale in SCALES {
        let width = style.width("12.5", scale);
        assert_eq!(width, width.ceil());
        let rects = all(&bars("12.5", scale, style));
        let right = rects.iter().map(|r| r.x + r.w).fold(0.0, f32::max);
        assert!(right <= width + 1.0, "at {scale}: {right} > {width}");
        assert!(right >= width - 2.0, "at {scale}: {right} < {width}");
    }
}

#[test]
fn a_longer_text_is_wider_and_a_bigger_scale_is_wider() {
    let style = Style::new(20.0);
    assert!(style.width("88", 1.0) > style.width("8", 1.0));
    assert!(style.width("8", 2.0) > style.width("8", 1.0));
}

fn extent(rects: &[Rect]) -> (f32, f32, f32, f32) {
    let left = rects.iter().map(|r| r.x).fold(f32::MAX, f32::min);
    let top = rects.iter().map(|r| r.y).fold(f32::MAX, f32::min);
    let right = rects.iter().map(|r| r.x + r.w).fold(0.0, f32::max);
    let bottom = rects.iter().map(|r| r.y + r.h).fold(0.0, f32::max);
    (left, top, right, bottom)
}

#[test]
fn the_text_is_placed_at_the_start_the_centre_or_the_end_of_the_area() {
    let area = Rect::new(100.0, 50.0, 300.0, 60.0);
    let style = Style::new(20.0);
    let place = |align| extent(&all(&layout("12.5", area, 1.0, align, style)));
    let (left, ..) = place(Align::Start);
    assert!((left - 100.0).abs() <= 1.0);
    let (.., right, _) = place(Align::End);
    assert!((right - 400.0).abs() <= 1.0);
    let (l, _, r, _) = place(Align::Center);
    assert!(((l - 100.0) - (400.0 - r)).abs() <= 2.0);
}

#[test]
fn the_digits_are_centred_up_and_down() {
    let area = Rect::new(0.0, 50.0, 300.0, 60.0);
    let (_, top, _, bottom) = extent(&all(&layout(
        "8",
        area,
        1.0,
        Align::Start,
        Style::new(20.0),
    )));
    assert!(((top - 50.0) - (110.0 - bottom)).abs() <= 1.0);
    assert!((bottom - top - 20.0).abs() <= 1.0);
}

#[test]
fn text_wider_than_the_area_overflows_instead_of_being_cut() {
    let area = Rect::new(10.0, 0.0, 20.0, 30.0);
    let b = layout("88888", area, 1.0, Align::Start, Style::new(20.0));
    let (left, _, right, _) = extent(&all(&b));
    assert!((left - 10.0).abs() <= 1.0);
    assert!(right > 30.0);
    assert_eq!(b.lit.len(), 35);
}

#[test]
fn the_dot_sits_on_the_baseline() {
    let style = Style::new(20.0);
    let digit = extent(&all(&bars("8", 1.0, style)));
    let dot = extent(&all(&bars(".", 1.0, style)));
    assert!((dot.3 - digit.3).abs() <= 1.0);
}
