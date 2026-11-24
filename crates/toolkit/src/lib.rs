//! Pure logic for custom-drawn controls. No UI, no drawing, no disk, no network.
//!
//! The window layer turns its events into calls on [`input::Input`] (where the pointer is,
//! which control is under it, button down, button up); controls such as [`widgets::Button`]
//! and [`widgets::Toggle`] turn the answers into clicks and into how they should look.
//! [`layout`] works out where things are (rows and columns, in points, scaled and snapped to
//! whole pixels). Which control is under the pointer comes from the caller's layout, so what is drawn and
//! what is hit stay the same thing.

pub mod geometry;
pub mod input;
pub mod knob;
pub mod layout;
pub mod slider;
pub mod value;
pub mod widgets;
