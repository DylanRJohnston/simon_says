use std::sync::LazyLock;

use bevy::color::Color;

pub const SCREEN_CONTAINER_PADDING: f32 = 16.;

pub const UI_CONTAINER_PADDING: f32 = 24.;
pub const UI_CONTAINER_RADIUS: f32 = 24.;
pub const UI_CONTAINER_GAP: f32 = 8.;

// https://lospec.com/palette-list/sweetie-16
pub static UI_BACKGROUND_COLOR: LazyLock<Color> =
    LazyLock::new(|| Color::srgb_u8(0x56, 0x6c, 0x86));

pub static PRIMARY_TEXT_COLOR: LazyLock<Color> = LazyLock::new(|| Color::srgb_u8(0xf4, 0xf4, 0xf4));
pub static GHOST_TEXT_COLOR: LazyLock<Color> = LazyLock::new(|| Color::srgb_u8(0x94, 0xb0, 0xc2));

// Button Constants
pub const BUTTON_BORDER_THICKNESS: f32 = 4.;
pub const BUTTON_BORDER_RADIUS: f32 = 8.;

// pub static BUTTON_COLOR: LazyLock<Color> = LazyLock::new(|| Color::srgb_u8(0x41, 0xa6, 0xf6));
pub static BUTTON_COLOR: LazyLock<Color> = LazyLock::new(|| Color::srgb_u8(0x2b, 0x7c, 0xbd));
pub static BUTTON_CANCEL_COLOR: LazyLock<Color> =
    LazyLock::new(|| Color::srgb_u8(0xb1, 0x3e, 0x53));
pub static BUTTON_DISABLED_COLOR: LazyLock<Color> =
    LazyLock::new(|| Color::srgb_u8(0x56, 0x6c, 0x86));
// pub static BUTTON_SUCCESS_COLOR: LazyLock<Color> =
//     LazyLock::new(|| Color::srgb_u8(0x38, 0xb7, 0x64));
pub static BUTTON_SUCCESS_COLOR: LazyLock<Color> =
    LazyLock::new(|| Color::srgb_u8(0x1a, 0x8f, 0x42));
