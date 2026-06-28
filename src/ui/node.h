#ifndef SKIA_YOGA_UI_NODE_H_
#define SKIA_YOGA_UI_NODE_H_

#include <functional>
#include <memory>
#include <string>
#include <vector>

#include "include/core/SkCanvas.h"
#include "include/core/SkColor.h"
#include "include/core/SkFont.h"
#include "include/core/SkPaint.h"
#include "include/core/SkTypeface.h"
#include "yoga/Yoga.h"

namespace skui::ui {

// Returns a platform-appropriate default typeface.
sk_sp<SkTypeface> GetDefaultTypeface();

enum class FlexDirection { Row, Column, RowReverse, ColumnReverse };

enum class Justify {
  FlexStart,
  Center,
  FlexEnd,
  SpaceBetween,
  SpaceAround,
  SpaceEvenly
};

enum class Align {
  Auto,
  FlexStart,
  Center,
  FlexEnd,
  Stretch,
  Baseline,
  SpaceBetween,
  SpaceAround
};

class Node : public std::enable_shared_from_this<Node> {
 public:
  Node();
  virtual ~Node();

  // Hierarchy
  void AddChild(std::shared_ptr<Node> child);
  const std::vector<std::shared_ptr<Node>>& GetChildren() const {
    return children_;
  }

  // Layout Styles
  void SetFlexDirection(FlexDirection direction);
  void SetJustifyContent(Justify justify);
  void SetAlignItems(Align align);
  void SetAlignSelf(Align align);

  void SetWidth(float width);
  void SetHeight(float height);
  void SetMinWidth(float width);
  void SetMinHeight(float height);
  void SetMaxWidth(float width);
  void SetMaxHeight(float height);

  void SetFlexGrow(float grow);
  void SetFlexShrink(float shrink);
  void SetFlexBasis(float basis);

  void SetPadding(YGEdge edge, float padding);
  void SetMargin(YGEdge edge, float margin);
  void SetPosition(YGEdge edge, float position);
  void SetPositionType(YGPositionType positionType);

  // Background and Border Styles
  void SetBackgroundColor(SkColor color) {
    bg_color_ = color;
    has_bg_ = true;
  }
  void SetBorder(float width, SkColor color) {
    border_width_ = width;
    border_color_ = color;
    has_border_ = true;
  }
  void SetCornerRadius(float radius) { corner_radius_ = radius; }
  void SetClipChildren(bool clip) { clips_children_ = clip; }

  // Layout Pass
  void CalculateLayout(float ownerWidth = YGUndefined,
                       float ownerHeight = YGUndefined,
                       YGDirection direction = YGDirectionLTR);

  // Get Layout Results
  float GetLeft() const;
  float GetTop() const;
  float GetWidth() const;
  float GetHeight() const;

  // Render Pass
  virtual void Render(SkCanvas* canvas);

 protected:
  virtual void OnDraw(SkCanvas* canvas, float width, float height) {}
  YGNodeRef yg_node_;

 private:
  std::vector<std::shared_ptr<Node>> children_;

  bool has_bg_ = false;
  SkColor bg_color_ = SK_ColorTRANSPARENT;

  bool has_border_ = false;
  float border_width_ = 0.0f;
  SkColor border_color_ = SK_ColorTRANSPARENT;
  float corner_radius_ = 0.0f;
  bool clips_children_ = false;
};

class TextNode : public Node {
 public:
  explicit TextNode(std::string text = "");
  ~TextNode() override;

  void SetText(std::string text);
  void SetFontSize(float size);
  void SetTextColor(SkColor color);
  void SetTypeface(sk_sp<SkTypeface> typeface);

  YGSize Measure(float width, YGMeasureMode widthMode, float height,
                 YGMeasureMode heightMode);

 protected:
  void OnDraw(SkCanvas* canvas, float width, float height) override;

 private:
  std::string text_;
  float font_size_ = 13.0f;
  SkColor text_color_ = SkColorSetRGB(0xF5, 0xF7, 0xFA);
  sk_sp<SkTypeface> typeface_;
};

}  // namespace skui::ui

#endif  // SKIA_YOGA_UI_NODE_H_
