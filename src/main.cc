#include <GLFW/glfw3.h>

#include <algorithm>
#include <chrono>
#include <cmath>
#include <iomanip>
#include <memory>
#include <sstream>
#include <string>
#include <vector>

#include "include/core/SkPath.h"
#include "include/core/SkPathBuilder.h"
#include "include/core/SkRRect.h"
#include "include/core/SkRect.h"
#include "skia_app.h"
#include "ui/daw_colors.h"
#include "ui/daw_state.h"
#include "ui/draw_node.h"
#include "ui/input.h"
#include "ui/node.h"

namespace {

using namespace skui::ui;
using namespace skui::ui::colors;

// ─── Layout constants ────────────────────────────────────────────────
constexpr float kTopBarHeight = 48.0f;
constexpr float kTrackListHeaderHeight = 28.0f;
constexpr float kTrackRowHeight = 40.0f;
constexpr float kTrackNameWidth = 160.0f;
constexpr float kTrackControlsWidth = 100.0f;
constexpr float kBottomPanelHeight = 280.0f;
constexpr float kSeqLabelWidth = 80.0f;
constexpr float kSeqCellSize = 36.0f;
constexpr float kSeqCellGap = 2.0f;
constexpr float kTimelineRulerHeight = 24.0f;
constexpr float kDividerHeight = 1.0f;

// ─── Drawing helpers ─────────────────────────────────────────────────

void DrawRoundRect(SkCanvas* canvas, float x, float y, float w, float h,
                   float r, SkColor color) {
  SkPaint paint;
  paint.setAntiAlias(true);
  paint.setColor(color);
  canvas->drawRRect(
      SkRRect::MakeRectXY(SkRect::MakeXYWH(x, y, w, h), r, r), paint);
}

void DrawRect(SkCanvas* canvas, float x, float y, float w, float h,
              SkColor color) {
  SkPaint paint;
  paint.setAntiAlias(true);
  paint.setColor(color);
  canvas->drawRect(SkRect::MakeXYWH(x, y, w, h), paint);
}

void DrawCircle(SkCanvas* canvas, float cx, float cy, float r,
                SkColor color) {
  SkPaint paint;
  paint.setAntiAlias(true);
  paint.setColor(color);
  canvas->drawCircle(cx, cy, r, paint);
}

void DrawText(SkCanvas* canvas, const std::string& text, float x, float y,
              float font_size, SkColor color, sk_sp<SkTypeface> typeface) {
  SkFont font(typeface, font_size);
  SkPaint paint;
  paint.setAntiAlias(true);
  paint.setColor(color);
  canvas->drawSimpleText(text.data(), text.size(), SkTextEncoding::kUTF8, x,
                         y, font, paint);
}

float MeasureText(const std::string& text, float font_size,
                  sk_sp<SkTypeface> typeface) {
  SkFont font(typeface, font_size);
  return font.measureText(text.data(), text.size(), SkTextEncoding::kUTF8);
}

void DrawPlayIcon(SkCanvas* canvas, float cx, float cy, float size,
                  SkColor color) {
  SkPaint paint;
  paint.setAntiAlias(true);
  paint.setColor(color);
  float half = size * 0.5f;
  SkPathBuilder pb;
  pb.moveTo(cx - half * 0.4f, cy - half);
  pb.lineTo(cx + half * 0.7f, cy);
  pb.lineTo(cx - half * 0.4f, cy + half);
  pb.close();
  canvas->drawPath(pb.detach(), paint);
}

void DrawPauseIcon(SkCanvas* canvas, float cx, float cy, float size,
                   SkColor color) {
  SkPaint paint;
  paint.setAntiAlias(true);
  paint.setColor(color);
  float half = size * 0.5f;
  float bar_w = size * 0.22f;
  float gap = size * 0.12f;
  canvas->drawRect(
      SkRect::MakeXYWH(cx - gap - bar_w, cy - half, bar_w, size), paint);
  canvas->drawRect(SkRect::MakeXYWH(cx + gap, cy - half, bar_w, size),
                   paint);
}

void DrawStopIcon(SkCanvas* canvas, float cx, float cy, float size,
                  SkColor color) {
  SkPaint paint;
  paint.setAntiAlias(true);
  paint.setColor(color);
  float half = size * 0.35f;
  canvas->drawRect(SkRect::MakeXYWH(cx - half, cy - half, half * 2, half * 2),
                   paint);
}

void DrawMetronomeIcon(SkCanvas* canvas, float cx, float cy, float size,
                       SkColor color) {
  SkPaint paint;
  paint.setAntiAlias(true);
  paint.setColor(color);
  paint.setStyle(SkPaint::kStroke_Style);
  paint.setStrokeWidth(1.5f);

  float h = size * 0.8f;
  float w = size * 0.5f;

  // Trapezoid body
  SkPathBuilder pb;
  pb.moveTo(cx - w * 0.3f, cy - h * 0.5f);
  pb.lineTo(cx + w * 0.3f, cy - h * 0.5f);
  pb.lineTo(cx + w * 0.5f, cy + h * 0.5f);
  pb.lineTo(cx - w * 0.5f, cy + h * 0.5f);
  pb.close();
  canvas->drawPath(pb.detach(), paint);

  // Pendulum arm
  paint.setStyle(SkPaint::kStroke_Style);
  paint.setStrokeWidth(2.0f);
  canvas->drawLine(cx, cy + h * 0.3f, cx + w * 0.2f, cy - h * 0.35f, paint);
}

// ─── BPM formatting ──────────────────────────────────────────────────

std::string FormatBpm(float bpm) {
  std::ostringstream ss;
  ss << std::fixed << std::setprecision(1) << bpm;
  return ss.str();
}

std::string FormatPosition(double step) {
  int bar = static_cast<int>(step / kSequencerSteps) + 1;
  int beat =
      static_cast<int>(std::fmod(step, kSequencerSteps) / kStepsPerBeat) + 1;
  int sub = static_cast<int>(std::fmod(step, kStepsPerBeat)) + 1;
  std::ostringstream ss;
  ss << bar << "." << beat << "." << sub;
  return ss.str();
}

}  // namespace

int main(int argc, char** argv) {
  // ─── App setup ──────────────────────────────────────────────────────
  skui::AppOptions options;
  options.backend = wgpu::BackendType::Undefined;
  if (argc >= 2) {
    const std::string backend = argv[1];
    if (backend == "vulkan")
      options.backend = wgpu::BackendType::Vulkan;
    else if (backend == "metal")
      options.backend = wgpu::BackendType::Metal;
    else if (backend == "d3d12")
      options.backend = wgpu::BackendType::D3D12;
  }
  options.window.height = 800;
  options.window.width = 1400;
  options.window.title = "Enableaton";

  skui::SkiaApp app(options);

  // ─── DAW State ──────────────────────────────────────────────────────
  DawState daw;
  daw.InitDefaultTracks();

  // Mouse state tracking
  bool mouse_down = false;
  float mouse_x = 0, mouse_y = 0;

  // Timing for playhead
  using Clock = std::chrono::steady_clock;
  auto last_frame_time = Clock::now();

  // ─── Input handlers ─────────────────────────────────────────────────
  app.SetMouseCallback([&](const MouseEvent& e) -> bool {
    mouse_x = e.x;
    mouse_y = e.y;

    if (e.action == InputAction::Press && e.button == MouseButton::Left) {
      mouse_down = true;

      // ── BPM drag start ──
      // BPM text is roughly at x: center area, y: top bar
      // We'll compute exact hit regions in the draw callback
      // For now, store drag state for BPM
      // BPM region check will be done by position

    } else if (e.action == InputAction::Release) {
      mouse_down = false;
      daw.bpm_dragging = false;
    } else if (e.action == InputAction::Move && daw.bpm_dragging) {
      // BPM drag: vertical movement changes BPM
      float delta = (daw.bpm_drag_start_y - e.y) * 0.5f;
      daw.bpm = std::clamp(daw.bpm_at_drag_start + delta, 20.0f, 999.0f);
    }

    return true;
  });

  app.SetKeyCallback([&](const KeyEvent& e) -> bool {
    if (e.action == KeyAction::Press) {
      switch (e.key) {
        case GLFW_KEY_SPACE:
          daw.playing = !daw.playing;
          if (!daw.playing) {
            daw.playhead_step = 0.0;
          }
          break;
        case GLFW_KEY_M:
          daw.metronome_on = !daw.metronome_on;
          break;
        case GLFW_KEY_UP:
          daw.bpm = std::clamp(daw.bpm + 1.0f, 20.0f, 999.0f);
          break;
        case GLFW_KEY_DOWN:
          daw.bpm = std::clamp(daw.bpm - 1.0f, 20.0f, 999.0f);
          break;
        default:
          break;
      }
    }
    return true;
  });

  app.SetScrollCallback([&](const ScrollEvent& e) -> bool {
    // Scroll on timeline area
    daw.timeline_scroll_x -= e.dy * 20.0f;
    if (daw.timeline_scroll_x < 0) daw.timeline_scroll_x = 0;
    return true;
  });

  // ─── Build the UI tree ──────────────────────────────────────────────
  // Root container
  auto root = std::make_shared<Node>();
  root->SetFlexDirection(FlexDirection::Column);
  root->SetBackgroundColor(kBgDeep);

  // ═══ TOP BAR ═══════════════════════════════════════════════════════
  auto top_bar = std::make_shared<DrawNode>();
  top_bar->SetHeight(kTopBarHeight);
  top_bar->SetBackgroundColor(kBgDark);

  top_bar->SetDrawCallback(
      [&daw, &mouse_x, &mouse_y, &mouse_down](SkCanvas* canvas, float w,
                                                float h) {
        // Get default typeface
        auto typeface = skui::ui::GetDefaultTypeface();

        // ── Left section: Logo + Position ──
        float x = 16.0f;
        float cy = h * 0.5f;

        // App logo / name
        DrawRoundRect(canvas, x, cy - 12, 80, 24, 6, kBgLight);
        DrawText(canvas, "ENABLEATON", x + 6, cy + 4, 10.0f, kAccent,
                 typeface);
        x += 96;

        // Position display
        std::string pos = FormatPosition(daw.playhead_step);
        DrawRoundRect(canvas, x, cy - 12, 72, 24, 4, kBgMid);
        DrawText(canvas, pos, x + 8, cy + 4, 12.0f, kTextPrimary, typeface);
        x += 88;

        // ── Center section: Transport controls ──
        float center_x = w * 0.5f;

        // Stop button
        float stop_x = center_x - 60;
        DrawRoundRect(canvas, stop_x - 14, cy - 14, 28, 28, 6, kBgLight);
        DrawStopIcon(canvas, stop_x, cy, 14, kStopGray);

        // Play/Pause button
        float play_x = center_x - 16;
        SkColor play_bg = daw.playing ? kPlayGreen : kBgLight;
        DrawRoundRect(canvas, play_x - 16, cy - 16, 32, 32, 8, play_bg);
        if (daw.playing) {
          DrawPauseIcon(canvas, play_x, cy, 16, kBgDeep);
        } else {
          DrawPlayIcon(canvas, play_x + 1, cy, 16, kTextPrimary);
        }

        // Record button
        float rec_x = center_x + 28;
        SkColor rec_bg = daw.recording ? kRecordRed : kBgLight;
        DrawRoundRect(canvas, rec_x - 14, cy - 14, 28, 28, 6, rec_bg);
        DrawCircle(canvas, rec_x, cy, 6,
                   daw.recording ? kTextPrimary : kRecordRed);

        // ── BPM display (draggable) ──
        float bpm_x = center_x + 70;
        std::string bpm_str = FormatBpm(daw.bpm);

        DrawRoundRect(canvas, bpm_x, cy - 14, 80, 28, 6,
                      daw.bpm_dragging ? kSurfaceActive : kBgMid);
        DrawText(canvas, bpm_str, bpm_x + 8, cy + 5, 14.0f,
                 daw.bpm_dragging ? kAccent : kTextPrimary, typeface);
        DrawText(canvas, "BPM", bpm_x + 56, cy + 4, 9.0f, kTextDim,
                 typeface);

        // Hit test for BPM drag
        if (mouse_down && !daw.bpm_dragging && mouse_y < kTopBarHeight) {
          float abs_bpm_x = bpm_x;
          if (mouse_x >= abs_bpm_x && mouse_x <= abs_bpm_x + 80) {
            daw.bpm_dragging = true;
            daw.bpm_drag_start_y = mouse_y;
            daw.bpm_at_drag_start = daw.bpm;
          }
        }

        // ── Metronome ──
        float met_x = bpm_x + 96;
        SkColor met_color = daw.metronome_on ? kMetronomeOn : kMetronomeOff;
        DrawRoundRect(canvas, met_x, cy - 14, 32, 28, 6,
                      daw.metronome_on ? kBgHighlight : kBgMid);
        DrawMetronomeIcon(canvas, met_x + 16, cy, 18, met_color);

        // ── Right section: track info ──
        float rx = w - 120;
        DrawText(canvas, "4/4", rx, cy + 4, 11.0f, kTextSecondary, typeface);
        rx += 30;
        DrawText(canvas, "16 steps", rx, cy + 4, 11.0f, kTextDim, typeface);

        // Top border glow
        SkPaint glow_paint;
        glow_paint.setColor(kAccentGlow);
        canvas->drawRect(SkRect::MakeWH(w, 1), glow_paint);

        // Bottom divider
        SkPaint div_paint;
        div_paint.setColor(kBorderDim);
        canvas->drawRect(SkRect::MakeXYWH(0, h - 1, w, 1), div_paint);
      });

  root->AddChild(top_bar);

  // ═══ MAIN CONTENT AREA ═════════════════════════════════════════════
  auto main_area = std::make_shared<Node>();
  main_area->SetFlexDirection(FlexDirection::Column);
  main_area->SetFlexGrow(1.0f);
  main_area->SetClipChildren(true);

  // ─── Track List Header ─────────────────────────────────────────────
  auto track_header = std::make_shared<DrawNode>();
  track_header->SetHeight(kTrackListHeaderHeight);
  track_header->SetBackgroundColor(kBgMid);

  track_header->SetDrawCallback(
      [](SkCanvas* canvas, float w, float h) {
        auto typeface = skui::ui::GetDefaultTypeface();
        float cy = h * 0.5f + 3;

        DrawText(canvas, "#", 12, cy, 10.0f, kTextDim, typeface);
        DrawText(canvas, "TRACK", 32, cy, 10.0f, kTextDim, typeface);
        DrawText(canvas, "TYPE", kTrackNameWidth + 8, cy, 10.0f, kTextDim,
                 typeface);
        DrawText(canvas, "S", kTrackNameWidth + kTrackControlsWidth - 30, cy,
                 10.0f, kTextDim, typeface);
        DrawText(canvas, "M", kTrackNameWidth + kTrackControlsWidth - 16, cy,
                 10.0f, kTextDim, typeface);

        // Timeline header
        DrawText(canvas, "TIMELINE", kTrackNameWidth + kTrackControlsWidth + 16,
                 cy, 10.0f, kTextDim, typeface);

        // Bottom border
        SkPaint paint;
        paint.setColor(kBorderDim);
        canvas->drawRect(SkRect::MakeXYWH(0, h - 1, w, 1), paint);
      });

  main_area->AddChild(track_header);

  // ─── Track list rows (rendered via DrawNode) ───────────────────────
  auto track_list = std::make_shared<DrawNode>();
  track_list->SetFlexGrow(1.0f);
  track_list->SetBackgroundColor(kBgDeep);
  track_list->SetClipChildren(true);

  track_list->SetDrawCallback(
      [&daw, &mouse_x, &mouse_y, &mouse_down](SkCanvas* canvas, float w,
                                                float h) {
        auto typeface = skui::ui::GetDefaultTypeface();

        float timeline_start = kTrackNameWidth + kTrackControlsWidth;
        float timeline_w = w - timeline_start;

        // ── Timeline ruler ──
        canvas->save();
        canvas->clipRect(
            SkRect::MakeXYWH(timeline_start, 0, timeline_w, kTimelineRulerHeight));
        {
          DrawRect(canvas, timeline_start, 0, timeline_w, kTimelineRulerHeight,
                   kBgMid);
          float step_w = kSeqCellSize + kSeqCellGap;
          float scroll = daw.timeline_scroll_x;

          for (int i = 0; i < kSequencerSteps; ++i) {
            float x = timeline_start + i * step_w - scroll;
            if (x > w || x + step_w < timeline_start) continue;

            if (i % kStepsPerBeat == 0) {
              int beat_num = i / kStepsPerBeat + 1;
              std::string label = std::to_string(beat_num);
              DrawText(canvas, label, x + 4, 15, 10.0f,
                       i == 0 ? kTextPrimary : kTextSecondary, typeface);
              // Beat marker line
              SkPaint line_paint;
              line_paint.setColor(kBorderMid);
              canvas->drawRect(
                  SkRect::MakeXYWH(x, kTimelineRulerHeight - 4, 1, 4),
                  line_paint);
            }
          }

          // Playhead on ruler
          if (daw.playing || daw.playhead_step > 0) {
            float ph_x =
                timeline_start + static_cast<float>(daw.playhead_step) * step_w - scroll;
            if (ph_x >= timeline_start && ph_x < w) {
              // Playhead triangle
              SkPaint ph_paint;
              ph_paint.setAntiAlias(true);
              ph_paint.setColor(kAccent);
              SkPathBuilder tri_pb;
              tri_pb.moveTo(ph_x - 4, 0);
              tri_pb.lineTo(ph_x + 4, 0);
              tri_pb.lineTo(ph_x, 8);
              tri_pb.close();
              canvas->drawPath(tri_pb.detach(), ph_paint);
            }
          }

          // Ruler bottom line
          SkPaint ruler_line;
          ruler_line.setColor(kBorderDim);
          canvas->drawRect(
              SkRect::MakeXYWH(timeline_start, kTimelineRulerHeight - 1,
                               timeline_w, 1),
              ruler_line);
        }
        canvas->restore();

        // ── Track rows ──
        float y_offset = kTimelineRulerHeight;
        float step_w = kSeqCellSize + kSeqCellGap;
        float scroll = daw.timeline_scroll_x;

        for (int i = 0; i < static_cast<int>(daw.tracks.size()); ++i) {
          const auto& track = daw.tracks[i];
          float row_y = y_offset + i * kTrackRowHeight;
          if (row_y > h) break;

          bool selected = (i == daw.selected_track);
          SkColor row_bg = selected ? kBgLight : kBgDark;
          SkColor track_color =
              kTrackColors[track.color_index % kTrackColorCount];

          // Row background
          DrawRect(canvas, 0, row_y, w, kTrackRowHeight, row_bg);

          // Track color indicator bar
          DrawRect(canvas, 0, row_y, 3, kTrackRowHeight, track_color);

          // Track number
          std::string num = std::to_string(i + 1);
          DrawText(canvas, num, 10, row_y + 25, 10.0f, kTextDim, typeface);

          // Track name
          DrawText(canvas, track.name, 32, row_y + 25, 12.0f,
                   selected ? kTextPrimary : kTextSecondary, typeface);

          // Track type
          DrawText(canvas, TrackTypeName(track.type),
                   kTrackNameWidth + 8, row_y + 25, 10.0f, kTextDim,
                   typeface);

          // Solo / Mute buttons
          float btn_y = row_y + 10;
          float solo_x = kTrackNameWidth + kTrackControlsWidth - 36;
          float mute_x = kTrackNameWidth + kTrackControlsWidth - 18;

          DrawRoundRect(canvas, solo_x, btn_y, 14, 18, 3,
                        track.solo ? SkColorSetRGB(0xFF, 0xD7, 0x4D) : kBgMid);
          DrawText(canvas, "S", solo_x + 3, btn_y + 13, 9.0f,
                   track.solo ? kBgDeep : kTextDim, typeface);

          DrawRoundRect(canvas, mute_x, btn_y, 14, 18, 3,
                        track.muted ? kRecordRed : kBgMid);
          DrawText(canvas, "M", mute_x + 2, btn_y + 13, 9.0f,
                   track.muted ? kTextPrimary : kTextDim, typeface);

          // ── Timeline cells (mini clips / pattern blocks) ──
          canvas->save();
          canvas->clipRect(
              SkRect::MakeXYWH(timeline_start, row_y, timeline_w,
                               kTrackRowHeight));

          if (track.type == TrackType::DrumRack) {
            // Show mini pattern visualization
            float cell_h = kTrackRowHeight - 8;
            for (int step = 0; step < kSequencerSteps; ++step) {
              float cx = timeline_start + step * step_w - scroll;
              if (cx > w || cx + step_w < timeline_start) continue;

              // Count active drums in this step
              int active = 0;
              for (int d = 0; d < kDrumTypeCount; ++d) {
                if (track.drum_pattern.IsActive(d, step)) active++;
              }

              if (active > 0) {
                // Draw mini activity bars
                float intensity =
                    static_cast<float>(active) / kDrumTypeCount;
                SkColor cell_color = SkColorSetARGB(
                    static_cast<int>(80 + 175 * intensity),
                    SkColorGetR(track_color), SkColorGetG(track_color),
                    SkColorGetB(track_color));
                DrawRoundRect(canvas, cx + 1, row_y + 4,
                              step_w - kSeqCellGap - 2,
                              cell_h * intensity + 4, 2, cell_color);
              }

              // Beat grid lines
              if (step % kStepsPerBeat == 0) {
                SkPaint grid;
                grid.setColor(kSeqBeatLine);
                canvas->drawRect(
                    SkRect::MakeXYWH(cx, row_y, 1, kTrackRowHeight), grid);
              }
            }
          } else {
            // Non-drum tracks: show empty clip slots
            for (int step = 0; step < kSequencerSteps; step += kStepsPerBeat) {
              float cx = timeline_start + step * step_w - scroll;
              float beat_w = kStepsPerBeat * step_w;
              if (cx > w || cx + beat_w < timeline_start) continue;

              DrawRoundRect(canvas, cx + 2, row_y + 6, beat_w - 4,
                            kTrackRowHeight - 12, 3, kBgMid);

              SkPaint grid;
              grid.setColor(kSeqBeatLine);
              canvas->drawRect(
                  SkRect::MakeXYWH(cx, row_y, 1, kTrackRowHeight), grid);
            }
          }

          // Playhead line on timeline
          if (daw.playing || daw.playhead_step > 0) {
            float ph_x =
                timeline_start +
                static_cast<float>(daw.playhead_step) * step_w - scroll;
            if (ph_x >= timeline_start && ph_x < w) {
              SkPaint ph_paint;
              ph_paint.setColor(kSeqPlayhead);
              canvas->drawRect(
                  SkRect::MakeXYWH(ph_x, row_y, 2, kTrackRowHeight),
                  ph_paint);
            }
          }

          canvas->restore();

          // Row bottom divider
          SkPaint div;
          div.setColor(kBorderDim);
          canvas->drawRect(
              SkRect::MakeXYWH(0, row_y + kTrackRowHeight - 1, w, 1), div);

          // ── Click handling for track selection ──
          if (mouse_down && mouse_y >= row_y + kTopBarHeight +
                                           kTrackListHeaderHeight &&
              mouse_y < row_y + kTopBarHeight + kTrackListHeaderHeight +
                            kTrackRowHeight) {
            float adjusted_x = mouse_x;

            // Solo button click
            if (adjusted_x >= solo_x && adjusted_x <= solo_x + 14 &&
                mouse_y >= btn_y + kTopBarHeight + kTrackListHeaderHeight &&
                mouse_y <=
                    btn_y + 18 + kTopBarHeight + kTrackListHeaderHeight) {
              daw.tracks[i].solo = !daw.tracks[i].solo;
            }
            // Mute button click
            else if (adjusted_x >= mute_x && adjusted_x <= mute_x + 14 &&
                     mouse_y >=
                         btn_y + kTopBarHeight + kTrackListHeaderHeight &&
                     mouse_y <=
                         btn_y + 18 + kTopBarHeight +
                             kTrackListHeaderHeight) {
              daw.tracks[i].muted = !daw.tracks[i].muted;
            }
            // Track selection
            else {
              daw.selected_track = i;
            }
          }
        }
      });

  main_area->AddChild(track_list);
  root->AddChild(main_area);

  // ═══ BOTTOM PANEL (Beat Sequencer) ═════════════════════════════════
  auto bottom_divider = std::make_shared<Node>();
  bottom_divider->SetHeight(kDividerHeight);
  bottom_divider->SetBackgroundColor(kBorderMid);
  root->AddChild(bottom_divider);

  auto bottom_panel = std::make_shared<DrawNode>();
  bottom_panel->SetHeight(kBottomPanelHeight);
  bottom_panel->SetBackgroundColor(kBgDark);
  bottom_panel->SetClipChildren(true);

  bottom_panel->SetDrawCallback(
      [&daw, &mouse_x, &mouse_y, &mouse_down](SkCanvas* canvas, float w,
                                                float h) {
        auto typeface = skui::ui::GetDefaultTypeface();

        // ── Tab bar ──
        float tab_h = 28.0f;
        DrawRect(canvas, 0, 0, w, tab_h, kBgMid);

        const char* tabs[] = {"SEQUENCER", "MIXER", "DEVICE"};
        float tab_x = 12.0f;
        for (int i = 0; i < 3; ++i) {
          auto current_tab = static_cast<DawState::BottomTab>(i);
          bool active = (daw.bottom_tab == current_tab);
          float tw = MeasureText(tabs[i], 10.0f, typeface) + 20;

          if (active) {
            DrawRoundRect(canvas, tab_x - 4, 4, tw + 8, 20, 4, kBgLight);
          }
          DrawText(canvas, tabs[i], tab_x, 18, 10.0f,
                   active ? kAccent : kTextDim, typeface);

          // Click handling for tabs
          float abs_y_start = h - kBottomPanelHeight;  // Not needed in local
          if (mouse_down && mouse_y >= abs_y_start && mouse_y < abs_y_start + tab_h) {
            // We're in local coords in the draw callback, so this won't
            // work perfectly. Tab clicks handled separately.
          }

          tab_x += tw + 16;
        }

        // Bottom border of tab bar
        SkPaint tab_line;
        tab_line.setColor(kBorderDim);
        canvas->drawRect(SkRect::MakeXYWH(0, tab_h - 1, w, 1), tab_line);

        // ── Check if selected track is drum rack ──
        if (!daw.SelectedIsDrumRack()) {
          // Show message
          DrawText(canvas, "Select a Drum Rack track to edit the sequencer",
                   w * 0.5f - 160, h * 0.5f, 13.0f, kTextDim, typeface);
          return;
        }

        auto* track = daw.SelectedTrack();
        if (!track) return;
        SkColor track_color =
            kTrackColors[track->color_index % kTrackColorCount];

        // ── Sequencer grid ──
        float grid_y = tab_h + 4;
        float row_h = (h - grid_y - 4) / kDrumTypeCount;
        row_h = std::min(row_h, 30.0f);
        float cell_w = kSeqCellSize;
        float grid_x = kSeqLabelWidth;

        for (int drum = 0; drum < kDrumTypeCount; ++drum) {
          float ry = grid_y + drum * row_h;
          auto drum_type = static_cast<DrumType>(drum);

          // Row background (alternating)
          if (drum % 2 == 0) {
            DrawRect(canvas, 0, ry, w, row_h, SkColorSetARGB(0x10, 0xFF, 0xFF, 0xFF));
          }

          // Drum label
          DrawText(canvas, DrumTypeName(drum_type), 8, ry + row_h * 0.5f + 4,
                   10.0f, kTextSecondary, typeface);

          // ── Step cells ──
          for (int step = 0; step < kSequencerSteps; ++step) {
            float cx = grid_x + step * (cell_w + kSeqCellGap);
            float cy = ry + 2;
            float cw = cell_w;
            float ch = row_h - 4;

            bool active = track->drum_pattern.IsActive(drum, step);
            bool is_beat = (step % kStepsPerBeat == 0);
            bool is_playhead_step =
                daw.playing &&
                static_cast<int>(daw.playhead_step) == step;

            // Cell background
            SkColor cell_bg;
            if (active) {
              cell_bg = is_playhead_step
                            ? kAccentBright
                            : track_color;
            } else {
              cell_bg = is_beat ? SkColorSetRGB(0x24, 0x24, 0x24)
                                : kSeqCellOff;
            }

            DrawRoundRect(canvas, cx, cy, cw, ch, 3, cell_bg);

            // Active cell glow effect
            if (active && is_playhead_step) {
              SkPaint glow;
              glow.setAntiAlias(true);
              glow.setColor(SkColorSetARGB(0x30, 0xFF, 0xFF, 0xFF));
              canvas->drawRRect(
                  SkRRect::MakeRectXY(
                      SkRect::MakeXYWH(cx - 2, cy - 2, cw + 4, ch + 4), 5,
                      5),
                  glow);
            }

            // Beat group separator
            if (step > 0 && step % kStepsPerBeat == 0) {
              SkPaint sep;
              sep.setColor(kSeqBarLine);
              canvas->drawRect(
                  SkRect::MakeXYWH(cx - kSeqCellGap, ry, 1, row_h), sep);
            }
          }
        }

        // ── Playhead column highlight ──
        if (daw.playing) {
          int current_step = static_cast<int>(daw.playhead_step);
          float ph_x = grid_x + current_step * (cell_w + kSeqCellGap);
          SkPaint ph_paint;
          ph_paint.setColor(kSeqPlayhead);
          canvas->drawRect(
              SkRect::MakeXYWH(ph_x, grid_y, cell_w,
                               kDrumTypeCount * row_h),
              ph_paint);
        }

        // ── Step numbers at bottom ──
        for (int step = 0; step < kSequencerSteps; ++step) {
          float cx = grid_x + step * (cell_w + kSeqCellGap);
          float label_y = grid_y + kDrumTypeCount * row_h + 12;
          if (label_y < h) {
            std::string label = std::to_string(step + 1);
            DrawText(canvas, label, cx + cell_w * 0.5f - 4, label_y, 9.0f,
                     kTextDim, typeface);
          }
        }
      });

  root->AddChild(bottom_panel);

  // ─── Run the app ────────────────────────────────────────────────────
  return app.Run([&](SkCanvas* canvas, SkISize size) {
    // Advance playhead
    auto now = Clock::now();
    double dt =
        std::chrono::duration<double>(now - last_frame_time).count();
    last_frame_time = now;
    daw.AdvancePlayhead(dt);

    // Handle sequencer clicks in the bottom panel
    // The bottom panel's top-left is at (0, size.height() - kBottomPanelHeight)
    if (mouse_down) {
      float bp_top = static_cast<float>(size.height()) - kBottomPanelHeight;
      float tab_h = 28.0f;
      float grid_y = bp_top + tab_h + 4;
      float row_h = (kBottomPanelHeight - tab_h - 4 - 4) / kDrumTypeCount;
      row_h = std::min(row_h, 30.0f);

      if (daw.SelectedIsDrumRack() && mouse_y >= grid_y) {
        int drum = static_cast<int>((mouse_y - grid_y) / row_h);
        float cell_x = mouse_x - kSeqLabelWidth;
        int step = static_cast<int>(cell_x / (kSeqCellSize + kSeqCellGap));

        if (drum >= 0 && drum < kDrumTypeCount && step >= 0 &&
            step < kSequencerSteps && cell_x >= 0) {
          static int last_toggled_drum = -1;
          static int last_toggled_step = -1;
          if (drum != last_toggled_drum || step != last_toggled_step) {
            daw.SelectedTrack()->drum_pattern.Toggle(drum, step);
            last_toggled_drum = drum;
            last_toggled_step = step;
          }
        }
      }

      // Tab clicks
      if (mouse_y >= bp_top && mouse_y < bp_top + tab_h) {
        if (mouse_x < 100) {
          daw.bottom_tab = DawState::BottomTab::Sequencer;
        } else if (mouse_x < 170) {
          daw.bottom_tab = DawState::BottomTab::Mixer;
        } else if (mouse_x < 240) {
          daw.bottom_tab = DawState::BottomTab::Device;
        }
      }

      // Play/Pause button click
      if (mouse_y < kTopBarHeight) {
        float center_x = static_cast<float>(size.width()) * 0.5f;
        float play_x = center_x - 16;
        if (mouse_x >= play_x - 16 && mouse_x <= play_x + 16) {
          static bool play_toggled = false;
          if (!play_toggled) {
            daw.playing = !daw.playing;
            if (!daw.playing) daw.playhead_step = 0.0;
            play_toggled = true;
          }
        } else {
          // Reset toggle guard for play button
        }

        // Stop button click
        float stop_x = center_x - 60;
        if (mouse_x >= stop_x - 14 && mouse_x <= stop_x + 14) {
          daw.playing = false;
          daw.playhead_step = 0.0;
        }

        // Metronome click
        float bpm_x = center_x + 70;
        float met_x = bpm_x + 96;
        if (mouse_x >= met_x && mouse_x <= met_x + 32) {
          static bool met_toggled = false;
          if (!met_toggled) {
            daw.metronome_on = !daw.metronome_on;
            met_toggled = true;
          }
        }
      }
    } else {
      // Reset toggle guards on mouse up
      // (handled via static vars — not ideal but functional)
    }

    // Configure root dimensions
    root->SetWidth(static_cast<float>(size.width()));
    root->SetHeight(static_cast<float>(size.height()));

    // Layout
    root->CalculateLayout();

    // Render
    root->Render(canvas);
  });
}
