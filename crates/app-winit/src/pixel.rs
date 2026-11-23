//! How a drawn pixel is handed to softbuffer, and where see-through windows are allowed.
//! Pure, so it is tested on its own. This is the one place that knows the pixel format.
//!
//! softbuffer 0.4.8 documents its pixel as 0RGB: the top byte is unused, and it has no
//! setting for transparency. On Windows (the spike, Windows 10) putting the alpha in the top
//! byte of a premultiplied pixel gave real see-through corners, but that is not something the
//! library promises, and nothing says it works elsewhere. So see-through is only used where it
//! has been seen to work, and everywhere else the window is opaque and still looks right.

/// Turns one tiny-skia pixel (premultiplied red, green, blue, alpha bytes) into the `u32`
/// softbuffer wants. Opaque windows drop the alpha. See-through windows put it in the top
/// byte, next to the premultiplied colour, which is what the Windows spike did.
pub fn pack(pixel: &[u8], see_through: bool) -> u32 {
    let (r, g, b) = (pixel[0] as u32, pixel[1] as u32, pixel[2] as u32);
    let rgb = (r << 16) | (g << 8) | b;
    if see_through {
        ((pixel[3] as u32) << 24) | rgb
    } else {
        rgb
    }
}

/// Whether a window that asked for see-through may have it. Only Windows has been tried;
/// `forced_opaque` (the `DOCK_OPAQUE` environment variable) turns it off there too, to see
/// the fallback.
pub fn see_through_supported(is_windows: bool, forced_opaque: bool) -> bool {
    is_windows && !forced_opaque
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_opaque_window_drops_the_alpha() {
        assert_eq!(pack(&[0x11, 0x22, 0x33, 0xFF], false), 0x0011_2233);
        assert_eq!(pack(&[0x11, 0x22, 0x33, 0x80], false), 0x0011_2233);
    }

    #[test]
    fn a_see_through_window_keeps_the_alpha_in_the_top_byte() {
        assert_eq!(pack(&[0x11, 0x22, 0x33, 0xFF], true), 0xFF11_2233);
        assert_eq!(pack(&[0x08, 0x10, 0x18, 0x80], true), 0x8008_1018);
    }

    #[test]
    fn a_fully_transparent_pixel_is_all_zero() {
        assert_eq!(pack(&[0, 0, 0, 0], true), 0);
    }

    #[test]
    fn see_through_is_only_allowed_where_it_has_been_seen_to_work() {
        assert!(see_through_supported(true, false));
        assert!(!see_through_supported(false, false));
    }

    #[test]
    fn it_can_be_forced_off() {
        assert!(!see_through_supported(true, true));
        assert!(!see_through_supported(false, true));
    }
}
