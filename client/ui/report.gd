extends PanelContainer
## Turn report (V-2a): after `step()`, the player's refused orders in red
## ("REJECTED: reason"), then the turn's events and the plain narration
## facts. The NES popups still carry the voice; this is the ledger view.
## Also the game-over screen (D105 #1).

const S := preload("res://ui/style.gd")

const CAUSES := {
	"Collapse": "REGIME COLLAPSE: stability ran out and the state came apart.",
	"Coup": "OVERTHROWN: the government fell to a coup.",
	"Extinct": "THE COUNTRY NO LONGER EXISTS.",
}

var body: VBoxContainer
var scroll: ScrollContainer


func setup() -> void:
	theme = S.theme()
	custom_minimum_size = Vector2(440, 0)
	var col := VBoxContainer.new()
	add_child(col)
	var top := HBoxContainer.new()
	col.add_child(top)
	var t := S.label("TURN REPORT", S.AMBER, 16)
	t.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	top.add_child(t)
	top.add_child(S.button("X", func() -> void: visible = false))
	scroll = ScrollContainer.new()
	scroll.custom_minimum_size = Vector2(420, 220)
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	col.add_child(scroll)
	body = VBoxContainer.new()
	body.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	scroll.add_child(body)
	visible = false


## Returns the plain-text lines it showed (the headless check prints them).
func show_turn(out: Dictionary, year: float) -> PackedStringArray:
	for c in body.get_children():
		c.queue_free()
	var lines := PackedStringArray()
	var add := func(text: String, color: Color) -> void:
		var l := S.label(text, color)
		l.custom_minimum_size = Vector2(400, 0)
		body.add_child(l)
		lines.append(text)
	add.call("%.2f  (resolved turn %d)" % [year, int(out.get("turn", 0))], S.AMBER)
	for r in out.get("rejected", []):
		add.call("REJECTED: %s — %s" % [r["text"], r["reason"]], S.RED)
	for n in out.get("narration", []):
		add.call("* " + String(n["fact"]), S.WHITE if int(n["gravity"]) >= 3 else S.HOLO)
	var evs: Array = out.get("events", [])
	if evs.size() > 0:
		var parts := []
		for e in evs:
			parts.append("%s %s%s" % [e["kind"], e["a"], (">" + String(e["b"])) if String(e["b"]) != "" else ""])
		add.call("events: " + ", ".join(parts), S.DIM)
	visible = true
	scroll.scroll_vertical = 0
	return lines


## Full-screen game over (regime collapse or coup).
static func game_over_screen(reason: String, year: float, cb_new: Callable) -> PanelContainer:
	var p := PanelContainer.new()
	p.theme = S.theme()
	p.add_theme_stylebox_override("panel", S.flat(Color(0, 0, 0, 0.95), S.RED, 4, 40))
	p.set_anchors_preset(Control.PRESET_FULL_RECT)
	var col := VBoxContainer.new()
	col.alignment = BoxContainer.ALIGNMENT_CENTER
	col.add_theme_constant_override("separation", 16)
	p.add_child(col)
	var title := S.label("GAME OVER", S.RED, 40)
	title.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	col.add_child(title)
	var cause := S.label(CAUSES.get(reason, "The government has ended (%s)." % reason), S.WHITE, 20)
	cause.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	col.add_child(cause)
	var when := S.label("%.2f" % year, S.AMBER, 18)
	when.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	col.add_child(when)
	var b := S.button("NEW GAME", cb_new)
	b.size_flags_horizontal = Control.SIZE_SHRINK_CENTER
	col.add_child(b)
	return p
