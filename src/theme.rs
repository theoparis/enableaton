// ── Palette (FL Studio 12-style flat gray-blue) ──────────────────────────────

/// Workspace background behind the docked windows.
pub const BG: u32 = 0x2a3136;
/// Window body.
pub const SURFACE: u32 = 0x353e45;
/// Raised controls / inactive buttons.
pub const SURFACE2: u32 = 0x464f57;
/// Recessed areas (lists, LCDs, grids).
pub const INSET: u32 = 0x22282c;
pub const BORDER: u32 = 0x1b2024;
/// Subtle highlight edge on raised controls.
pub const BEVEL: u32 = 0x56616a;
pub const TITLE_TOP: u32 = 0x56606a;
pub const TITLE_BOTTOM: u32 = 0x47515a;

pub const ACCENT: u32 = 0xf7a64a;
pub const ACCENT_DIM: u32 = 0x8a5f2c;
pub const LED_ON: u32 = 0x9ee35c;
pub const LED_OFF: u32 = 0x3a4247;
pub const TEXT: u32 = 0xc9d2d8;
pub const TEXT_BRIGHT: u32 = 0xeef3f6;
pub const TEXT_DIM: u32 = 0x86939b;
pub const RED: u32 = 0xe8494f;
pub const PLAYHEAD: u32 = 0x9ee35c;

pub const LCD_BG: u32 = 0x161a1d;
pub const LCD_TEXT: u32 = 0xd3e9f5;

/// Playlist grid: alternating 4-bar shading.
pub const GRID_A: u32 = 0x3b4851;
pub const GRID_B: u32 = 0x36424a;
pub const GRID_LINE: u32 = 0x2c363d;

/// Channel-rack step buttons (FL alternates gray/red groups of four).
pub const STEP_OFF_A: u32 = 0x5d656c;
pub const STEP_OFF_B: u32 = 0x6d5a5f;
pub const STEP_ON: u32 = 0xdfe4e8;

pub const TRACK_COLORS: &[u32] = &[
    0x6a5fc1, 0x4c7bc4, 0x3a9a8c, 0x5f9f45, 0xc4793a, 0xb8484f, 0xa8509c,
];

// ── Layout ───────────────────────────────────────────────────────────────────

pub const TOOLBAR_H: f32 = 46.0;
pub const WINDOW_TITLE_H: f32 = 22.0;
pub const BROWSER_W: f32 = 176.0;
pub const MID_COL_W: f32 = 476.0;
pub const MIXER_H: f32 = 236.0;
pub const RACK_H: f32 = 250.0;

pub const HEADER_W: f32 = 120.0;
pub const RULER_H: f32 = 22.0;
pub const TRACK_H: f32 = 52.0;
pub const CLIP_TITLE_H: f32 = 13.0;
pub const BEAT_PX: f32 = 20.0;
/// Rows always drawn in the playlist (empty tracks fill the rest, FL-style).
pub const MIN_PLAYLIST_ROWS: usize = 10;

pub const STEPS: usize = 16;
pub const CHANNEL_ROW_H: f32 = 24.0;
pub const STEP_BTN: f32 = 14.0;
pub const STEP_GAP: f32 = 2.0;
pub const STEP_GROUP_GAP: f32 = 4.0;

pub const MIXER_STRIP_W: f32 = 40.0;
pub const MIN_MIXER_INSERTS: usize = 28;

/// Lighten a 0xRRGGBB color towards white by `amt` (0..1).
pub fn lighten(color: u32, amt: f32) -> u32 {
    mix(color, 0xffffff, amt)
}

/// Darken a 0xRRGGBB color towards black by `amt` (0..1).
pub fn darken(color: u32, amt: f32) -> u32 {
    mix(color, 0x000000, amt)
}

pub fn mix(a: u32, b: u32, t: f32) -> u32 {
    let lerp = |s: u32| -> u32 {
        let x = ((a >> s) & 0xff) as f32;
        let y = ((b >> s) & 0xff) as f32;
        ((x + (y - x) * t).round() as u32).min(255) << s
    };
    lerp(16) | lerp(8) | lerp(0)
}
