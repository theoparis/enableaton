#ifndef SKIA_YOGA_UI_DAW_COLORS_H_
#define SKIA_YOGA_UI_DAW_COLORS_H_

#include "include/core/SkColor.h"

namespace skui::ui::colors {

// Background hierarchy
constexpr SkColor kBgDeep = SkColorSetRGB(0x0D, 0x0D, 0x0D);   // Deepest bg
constexpr SkColor kBgDark = SkColorSetRGB(0x14, 0x14, 0x14);   // Main bg
constexpr SkColor kBgMid = SkColorSetRGB(0x1A, 0x1A, 0x1A);    // Panel bg
constexpr SkColor kBgLight = SkColorSetRGB(0x22, 0x22, 0x22);  // Elevated bg
constexpr SkColor kBgHighlight =
    SkColorSetRGB(0x2A, 0x2A, 0x2A);  // Hover/selected bg

// Surface / Cards
constexpr SkColor kSurface = SkColorSetRGB(0x1E, 0x1E, 0x1E);
constexpr SkColor kSurfaceHover = SkColorSetRGB(0x28, 0x28, 0x28);
constexpr SkColor kSurfaceActive = SkColorSetRGB(0x30, 0x30, 0x30);

// Borders
constexpr SkColor kBorderDim = SkColorSetRGB(0x2E, 0x2E, 0x2E);
constexpr SkColor kBorderMid = SkColorSetRGB(0x3A, 0x3A, 0x3A);
constexpr SkColor kBorderBright = SkColorSetRGB(0x50, 0x50, 0x50);

// Text
constexpr SkColor kTextPrimary = SkColorSetRGB(0xE8, 0xE8, 0xE8);
constexpr SkColor kTextSecondary = SkColorSetRGB(0x9A, 0x9A, 0x9A);
constexpr SkColor kTextDim = SkColorSetRGB(0x66, 0x66, 0x66);
constexpr SkColor kTextAccent =
    SkColorSetRGB(0xFF, 0x76, 0x4D);  // Orange accent

// Accent colors (Ableton-inspired warm orange)
constexpr SkColor kAccent = SkColorSetRGB(0xFF, 0x76, 0x4D);  // Primary accent
constexpr SkColor kAccentDim =
    SkColorSetRGB(0xCC, 0x5E, 0x3D);  // Dimmed accent
constexpr SkColor kAccentBright =
    SkColorSetRGB(0xFF, 0x9A, 0x76);  // Bright accent
constexpr SkColor kAccentGlow = SkColorSetARGB(0x40, 0xFF, 0x76, 0x4D);  // Glow

// Transport colors
constexpr SkColor kPlayGreen = SkColorSetRGB(0x00, 0xD4, 0x6A);
constexpr SkColor kStopGray = SkColorSetRGB(0x88, 0x88, 0x88);
constexpr SkColor kRecordRed = SkColorSetRGB(0xFF, 0x3B, 0x3B);

// Sequencer colors
constexpr SkColor kSeqCellOff = SkColorSetRGB(0x1C, 0x1C, 0x1C);
constexpr SkColor kSeqCellOn = SkColorSetRGB(0xFF, 0x76, 0x4D);
constexpr SkColor kSeqCellOnDim = SkColorSetARGB(0x80, 0xFF, 0x76, 0x4D);
constexpr SkColor kSeqCellHover = SkColorSetRGB(0x33, 0x33, 0x33);
constexpr SkColor kSeqPlayhead = SkColorSetARGB(0x60, 0xFF, 0xFF, 0xFF);
constexpr SkColor kSeqBeatLine = SkColorSetRGB(0x2A, 0x2A, 0x2A);
constexpr SkColor kSeqBarLine = SkColorSetRGB(0x3A, 0x3A, 0x3A);

// Track colors (assignable per-track)
constexpr SkColor kTrackColors[] = {
    SkColorSetRGB(0xFF, 0x76, 0x4D),  // Orange
    SkColorSetRGB(0x4D, 0xB8, 0xFF),  // Blue
    SkColorSetRGB(0x4D, 0xFF, 0x91),  // Green
    SkColorSetRGB(0xFF, 0x4D, 0x8F),  // Pink
    SkColorSetRGB(0xC8, 0x7D, 0xFF),  // Purple
    SkColorSetRGB(0xFF, 0xD7, 0x4D),  // Yellow
    SkColorSetRGB(0x4D, 0xFF, 0xDB),  // Cyan
    SkColorSetRGB(0xFF, 0x4D, 0x4D),  // Red
};
constexpr int kTrackColorCount = sizeof(kTrackColors) / sizeof(kTrackColors[0]);

// Metronome
constexpr SkColor kMetronomeOn = SkColorSetRGB(0xFF, 0xD7, 0x4D);
constexpr SkColor kMetronomeOff = SkColorSetRGB(0x66, 0x66, 0x66);

}  // namespace skui::ui::colors

#endif  // SKIA_YOGA_UI_DAW_COLORS_H_
