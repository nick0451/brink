extends PanelContainer
## Start screen (V-2a): pick a country from `playable()` or spectate.
## Emits `chosen(code)`; "" means spectate (the AI-vs-AI mode).

signal chosen(code: String)

const S := preload("res://ui/style.gd")

var error_label: Label


func setup(playable: Array) -> void:
	theme = S.theme()
	add_theme_stylebox_override("panel", S.box())
	custom_minimum_size = Vector2(520, 0)
	var col := VBoxContainer.new()
	col.add_theme_constant_override("separation", 10)
	add_child(col)
	col.add_child(S.label("B R I N K", S.AMBER, 32))
	col.add_child(S.label("1980-2000. Choose a government to run, or watch the AI cabinets.", S.HOLO, 14))
	col.add_child(S.header("Assume office"))
	var grid := GridContainer.new()
	grid.columns = 2
	col.add_child(grid)
	for p in playable:
		var code: String = p["code"]
		var b := S.button("%s  %s" % [code, String(p["name"]).to_upper()], _pick.bind(code))
		b.disabled = not bool(p["selectable"])
		if b.disabled:
			b.tooltip_text = "Not selectable in this build"
		b.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		grid.add_child(b)
	col.add_child(S.header("Or"))
	col.add_child(S.button("SPECTATE  (the cabinets play each other)", _pick.bind("")))
	error_label = S.label("", S.RED)
	col.add_child(error_label)


func _ready() -> void:
	# Centre on screen once the size is known.
	await get_tree().process_frame
	position = (get_viewport_rect().size - size) * 0.5


func show_error(text: String) -> void:
	error_label.text = "REFUSED: " + text


func _pick(code: String) -> void:
	chosen.emit(code)
