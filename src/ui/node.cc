#include "node.h"

#include <iostream>

#include "include/core/SkFontMgr.h"
#include "include/core/SkPaint.h"
#include "include/core/SkRRect.h"
#include "include/core/SkRect.h"

#if defined(__APPLE__)
#include "include/ports/SkFontMgr_mac_ct.h"
#else
#include "include/ports/SkFontMgr_fontconfig.h"
#include "include/ports/SkFontScanner_FreeType.h"
#endif

namespace skui::ui {

sk_sp<SkTypeface> GetDefaultTypeface() {
#if defined(__APPLE__)
  static sk_sp<SkTypeface> typeface =
      SkFontMgr_New_CoreText(nullptr)->legacyMakeTypeface(nullptr,
                                                          SkFontStyle());
#else
  static sk_sp<SkTypeface> typeface =
      SkFontMgr_New_FontConfig(nullptr, SkFontScanner_Make_FreeType())
          ->legacyMakeTypeface(nullptr, SkFontStyle());
#endif
  return typeface;
}

namespace {

YGSize TextNodeMeasureFunc(YGNodeRef node, float width, YGMeasureMode widthMode,
                           float height, YGMeasureMode heightMode) {
  auto* textNode = static_cast<TextNode*>(YGNodeGetContext(node));
  return textNode->Measure(width, widthMode, height, heightMode);
}

}  // namespace

Node::Node() { yg_node_ = YGNodeNew(); }

Node::~Node() { YGNodeFree(yg_node_); }

void Node::AddChild(std::shared_ptr<Node> child) {
  children_.push_back(child);
  YGNodeInsertChild(yg_node_, child->yg_node_, YGNodeGetChildCount(yg_node_));
}

void Node::SetFlexDirection(FlexDirection direction) {
  YGFlexDirection yg_dir = YGFlexDirectionColumn;
  switch (direction) {
    case FlexDirection::Row:
      yg_dir = YGFlexDirectionRow;
      break;
    case FlexDirection::Column:
      yg_dir = YGFlexDirectionColumn;
      break;
    case FlexDirection::RowReverse:
      yg_dir = YGFlexDirectionRowReverse;
      break;
    case FlexDirection::ColumnReverse:
      yg_dir = YGFlexDirectionColumnReverse;
      break;
  }
  YGNodeStyleSetFlexDirection(yg_node_, yg_dir);
}

void Node::SetJustifyContent(Justify justify) {
  YGJustify yg_justify = YGJustifyFlexStart;
  switch (justify) {
    case Justify::FlexStart:
      yg_justify = YGJustifyFlexStart;
      break;
    case Justify::Center:
      yg_justify = YGJustifyCenter;
      break;
    case Justify::FlexEnd:
      yg_justify = YGJustifyFlexEnd;
      break;
    case Justify::SpaceBetween:
      yg_justify = YGJustifySpaceBetween;
      break;
    case Justify::SpaceAround:
      yg_justify = YGJustifySpaceAround;
      break;
    case Justify::SpaceEvenly:
      yg_justify = YGJustifySpaceEvenly;
      break;
  }
  YGNodeStyleSetJustifyContent(yg_node_, yg_justify);
}

void Node::SetAlignItems(Align align) {
  YGAlign yg_align = YGAlignAuto;
  switch (align) {
    case Align::Auto:
      yg_align = YGAlignAuto;
      break;
    case Align::FlexStart:
      yg_align = YGAlignFlexStart;
      break;
    case Align::Center:
      yg_align = YGAlignCenter;
      break;
    case Align::FlexEnd:
      yg_align = YGAlignFlexEnd;
      break;
    case Align::Stretch:
      yg_align = YGAlignStretch;
      break;
    case Align::Baseline:
      yg_align = YGAlignBaseline;
      break;
    case Align::SpaceBetween:
      yg_align = YGAlignSpaceBetween;
      break;
    case Align::SpaceAround:
      yg_align = YGAlignSpaceAround;
      break;
  }
  YGNodeStyleSetAlignItems(yg_node_, yg_align);
}

void Node::SetAlignSelf(Align align) {
  YGAlign yg_align = YGAlignAuto;
  switch (align) {
    case Align::Auto:
      yg_align = YGAlignAuto;
      break;
    case Align::FlexStart:
      yg_align = YGAlignFlexStart;
      break;
    case Align::Center:
      yg_align = YGAlignCenter;
      break;
    case Align::FlexEnd:
      yg_align = YGAlignFlexEnd;
      break;
    case Align::Stretch:
      yg_align = YGAlignStretch;
      break;
    case Align::Baseline:
      yg_align = YGAlignBaseline;
      break;
    case Align::SpaceBetween:
      yg_align = YGAlignSpaceBetween;
      break;
    case Align::SpaceAround:
      yg_align = YGAlignSpaceAround;
      break;
  }
  YGNodeStyleSetAlignSelf(yg_node_, yg_align);
}

void Node::SetWidth(float width) { YGNodeStyleSetWidth(yg_node_, width); }

void Node::SetHeight(float height) { YGNodeStyleSetHeight(yg_node_, height); }

void Node::SetMinWidth(float width) { YGNodeStyleSetMinWidth(yg_node_, width); }

void Node::SetMinHeight(float height) {
  YGNodeStyleSetMinHeight(yg_node_, height);
}

void Node::SetMaxWidth(float width) { YGNodeStyleSetMaxWidth(yg_node_, width); }

void Node::SetMaxHeight(float height) {
  YGNodeStyleSetMaxHeight(yg_node_, height);
}

void Node::SetFlexGrow(float grow) { YGNodeStyleSetFlexGrow(yg_node_, grow); }

void Node::SetFlexShrink(float shrink) {
  YGNodeStyleSetFlexShrink(yg_node_, shrink);
}

void Node::SetFlexBasis(float basis) {
  YGNodeStyleSetFlexBasis(yg_node_, basis);
}

void Node::SetPadding(YGEdge edge, float padding) {
  YGNodeStyleSetPadding(yg_node_, edge, padding);
}

void Node::SetMargin(YGEdge edge, float margin) {
  YGNodeStyleSetMargin(yg_node_, edge, margin);
}

void Node::SetPosition(YGEdge edge, float position) {
  YGNodeStyleSetPosition(yg_node_, edge, position);
}

void Node::SetPositionType(YGPositionType positionType) {
  YGNodeStyleSetPositionType(yg_node_, positionType);
}

void Node::CalculateLayout(float ownerWidth, float ownerHeight,
                           YGDirection direction) {
  YGNodeCalculateLayout(yg_node_, ownerWidth, ownerHeight, direction);
}

float Node::GetLeft() const { return YGNodeLayoutGetLeft(yg_node_); }

float Node::GetTop() const { return YGNodeLayoutGetTop(yg_node_); }

float Node::GetWidth() const { return YGNodeLayoutGetWidth(yg_node_); }

float Node::GetHeight() const { return YGNodeLayoutGetHeight(yg_node_); }

void Node::Render(SkCanvas* canvas) {
  const float width = GetWidth();
  const float height = GetHeight();

  // Save canvas matrix/clip
  canvas->save();
  canvas->translate(GetLeft(), GetTop());

  // Clip children if requested
  if (clips_children_) {
    canvas->clipRect(SkRect::MakeWH(width, height));
  }

  // Render Background
  if (has_bg_) {
    SkPaint paint;
    paint.setAntiAlias(true);
    paint.setColor(bg_color_);
    if (corner_radius_ > 0.0f) {
      canvas->drawRRect(SkRRect::MakeRectXY(SkRect::MakeWH(width, height),
                                            corner_radius_, corner_radius_),
                        paint);
    } else {
      canvas->drawRect(SkRect::MakeWH(width, height), paint);
    }
  }

  // Render Border
  if (has_border_ && border_width_ > 0.0f) {
    SkPaint paint;
    paint.setAntiAlias(true);
    paint.setColor(border_color_);
    paint.setStyle(SkPaint::kStroke_Style);
    paint.setStrokeWidth(border_width_);
    if (corner_radius_ > 0.0f) {
      canvas->drawRRect(SkRRect::MakeRectXY(SkRect::MakeWH(width, height),
                                            corner_radius_, corner_radius_),
                        paint);
    } else {
      canvas->drawRect(SkRect::MakeWH(width, height), paint);
    }
  }

  // Custom draw
  OnDraw(canvas, width, height);

  // Render Children
  for (auto& child : children_) {
    child->Render(canvas);
  }

  canvas->restore();
}

TextNode::TextNode(std::string text) : text_(std::move(text)) {
  typeface_ = GetDefaultTypeface();
  YGNodeSetContext(yg_node_, this);
  YGNodeSetMeasureFunc(yg_node_, TextNodeMeasureFunc);
}

TextNode::~TextNode() = default;

void TextNode::SetText(std::string text) {
  text_ = std::move(text);
  YGNodeMarkDirty(yg_node_);
}

void TextNode::SetFontSize(float size) {
  font_size_ = size;
  YGNodeMarkDirty(yg_node_);
}

void TextNode::SetTextColor(SkColor color) { text_color_ = color; }

void TextNode::SetTypeface(sk_sp<SkTypeface> typeface) {
  typeface_ = std::move(typeface);
  YGNodeMarkDirty(yg_node_);
}

YGSize TextNode::Measure(float width, YGMeasureMode widthMode, float height,
                         YGMeasureMode heightMode) {
  SkFont font(typeface_, font_size_);
  float text_width =
      font.measureText(text_.data(), text_.size(), SkTextEncoding::kUTF8);
  float text_height = font.getSize();
  return YGSize{text_width, text_height};
}

void TextNode::OnDraw(SkCanvas* canvas, float width, float height) {
  if (text_.empty()) {
    return;
  }
  SkFont font(typeface_, font_size_);
  SkPaint paint;
  paint.setAntiAlias(true);
  paint.setColor(text_color_);

  // Center vertically
  const float baseline = (height + font_size_) * 0.5f - 1.0f;

  canvas->drawSimpleText(text_.data(), text_.size(), SkTextEncoding::kUTF8,
                         0.0f, baseline, font, paint);
}

}  // namespace skui::ui
