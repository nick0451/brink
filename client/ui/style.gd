extends RefCounted
## Shared retro look for the V-2a panels (phosphor green on black, NES
## borders). Static helpers only; panels preload this script.

const HOLO := Color(0.25, 1.0, 0.65)
const DIM := Color(0.15, 0.6, 0.4)
const AMBER := Color(1.0, 0.8, 0.2)
const RED := Color(1.0, 0.3, 0.25)
const WHITE := Color(0.95, 0.95, 0.95)

static var _font: SystemFont
static var _theme: Theme


static func font() -> Font:
	if _font == null:
		_font = SystemFont.new()
		_font.font_names = PackedStringArray(["Lucida Console", "Consolas", "Courier New"])
		_font.antialiasing = TextServer.FONT_ANTIALIASING_NONE
	return _font


static func flat(bg: Color, border: Color, width: int, margin := 6) -> StyleBoxFlat:
	var sb := StyleBoxFlat.new()
	sb.bg_color = bg
	sb.border_color = border
	sb.set_border_width_all(width)
	sb.set_content_margin_all(margin)
	return sb


static func box(border := WHITE) -> StyleBoxFlat:
	return flat(Color(0, 0, 0, 0.9), border, 3, 10)


static func theme() -> Theme:
	if _theme != null:
		return _theme
	var t := Theme.new()
	t.default_font = font()
	t.default_font_size = 14
	t.set_color("font_color", "Label", HOLO)
	for cls in ["Button", "OptionButton", "CheckButton", "MenuButton"]:
		t.set_color("font_color", cls, HOLO)
		t.set_color("font_hover_color", cls, WHITE)
		t.set_color("font_pressed_color", cls, AMBER)
		t.set_color("font_focus_color", cls, HOLO)
		t.set_color("font_hover_pressed_color", cls, AMBER)
		t.set_color("font_disabled_color", cls, Color(0.3, 0.4, 0.35))
		t.set_stylebox("normal", cls, flat(Color(0, 0.08, 0.06), DIM, 1, 4))
		t.set_stylebox("hover", cls, flat(Color(0, 0.18, 0.12), HOLO, 1, 4))
		t.set_stylebox("pressed", cls, flat(Color(0.12, 0.1, 0), AMBER, 1, 4))
		t.set_stylebox("hover_pressed", cls, flat(Color(0.12, 0.1, 0), AMBER, 1, 4))
		t.set_stylebox("disabled", cls, flat(Color(0, 0.03, 0.02), Color(0.1, 0.25, 0.2), 1, 4))
		t.set_stylebox("focus", cls, StyleBoxEmpty.new())
	t.set_stylebox("panel", "PanelContainer", box())
	t.set_stylebox("panel", "PopupMenu", box(DIM))
	t.set_color("font_color", "PopupMenu", HOLO)
	t.set_color("font_hover_color", "PopupMenu", WHITE)
	t.set_stylebox("hover", "PopupMenu", flat(Color(0, 0.2, 0.12), HOLO, 1, 2))
	t.set_color("font_color", "LineEdit", HOLO)
	t.set_stylebox("normal", "LineEdit", flat(Color(0, 0.08, 0.06), DIM, 1, 2))
	t.set_stylebox("focus", "LineEdit", flat(Color(0, 0.08, 0.06), HOLO, 1, 2))
	t.set_stylebox("slider", "HSlider", flat(Color(0, 0.15, 0.1), DIM, 1, 2))
	t.set_stylebox("grabber_area", "HSlider", flat(HOLO * 0.6, HOLO * 0.6, 0, 2))
	t.set_stylebox("grabber_area_highlight", "HSlider", flat(HOLO, HOLO, 0, 2))
	_theme = t
	return t


static func label(text: String, color := HOLO, size := 14) -> Label:
	var l := Label.new()
	l.text = text
	l.add_theme_color_override("font_color", color)
	if size != 14:
		l.add_theme_font_size_override("font_size", size)
	l.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	l.mouse_filter = Control.MOUSE_FILTER_PASS
	return l


static func header(text: String) -> Label:
	var l := label("== %s ==" % text.to_upper(), AMBER, 14)
	return l


static func button(text: String, cb: Callable) -> Button:
	var b := Button.new()
	b.text = text
	b.focus_mode = Control.FOCUS_NONE  # Space must stay "end turn"
	b.pressed.connect(cb)
	return b


static func options(items: Array, selected := 0) -> OptionButton:
	var o := OptionButton.new()
	o.focus_mode = Control.FOCUS_NONE
	for it in items:
		o.add_item(str(it))
	if items.size() > 0:
		o.select(clampi(selected, 0, items.size() - 1))
	return o


static func row(children: Array) -> HBoxContainer:
	var h := HBoxContainer.new()
	h.add_theme_constant_override("separation", 6)
	for c in children:
		h.add_child(c)
	return h


## An estimate `[v, low, high]` as text, or "unknown" for null.
static func est(x: Variant, fmt := "%.0f") -> String:
	if x == null:
		return "unknown"
	if x is Array and x.size() == 3:
		return (fmt + " [" + fmt + "-" + fmt + "]") % [x[0], x[1], x[2]]
	if x is float or x is int:
		return fmt % x
	return str(x)


static func opt(x: Variant, fmt := "%.0f") -> String:
	if x == null:
		return "unknown"
	if x is float or x is int:
		return fmt % x
	return str(x)
