extends PanelContainer
## Country panel (V-2a). Click a capital on the map to show it here.
## Player mode: foreign countries come from `foreign(code)` (the player's
## own fog-filtered estimates, `[value, low, high]`, "unknown" when the
## coverage can't see them); the player's own country from `player_state()`.
## Spectator mode: the omniscient `countries()` row.

signal target_chosen(code: String)

const S := preload("res://ui/style.gd")

var sim: Node
var player := ""
var code := ""
var title: Label
var body: Label
var target_btn: Button


func setup(sim_node: Node, player_code: String) -> void:
	sim = sim_node
	player = player_code
	theme = S.theme()
	custom_minimum_size = Vector2(330, 0)
	var col := VBoxContainer.new()
	col.add_theme_constant_override("separation", 6)
	add_child(col)
	var top := HBoxContainer.new()
	col.add_child(top)
	title = S.label("", S.AMBER, 16)
	title.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	top.add_child(title)
	top.add_child(S.button("X", func() -> void: visible = false))
	var scroll := ScrollContainer.new()
	scroll.custom_minimum_size = Vector2(316, 380)
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	col.add_child(scroll)
	body = S.label("")
	body.custom_minimum_size = Vector2(300, 0)
	scroll.add_child(body)
	target_btn = S.button("SET AS ORDER TARGET", func() -> void: target_chosen.emit(code))
	col.add_child(target_btn)
	visible = false


func show_code(c: String) -> void:
	code = c
	visible = true
	refresh()


func refresh() -> void:
	if code == "" or sim == null:
		return
	target_btn.visible = player != "" and code != player
	if player == "":
		_spectator()
	elif code == player:
		_own(sim.player_state())
	else:
		_foreign(sim.foreign(code))


func _spectator() -> void:
	for c in sim.countries():
		if c["code"] == code:
			title.text = "%s  %s" % [code, String(c["name"]).to_upper()]
			body.text = "\n".join([
				"Government  %s (%s)" % [c["government"], c["alignment"]],
				"GDP         %.0f" % float(c["gdp"]),
				"Stability   %.0f" % float(c["stability"]),
				"Power       %.0f" % float(c["power"]),
				"Arsenal     %d" % int(c["arsenal"]),
				"At war      %s" % ("YES" if c["at_war"] else "no"),
			])
			return
	title.text = code
	body.text = "No such government. Possibly no longer."


func _own(p: Dictionary) -> void:
	if p.is_empty():
		return
	title.text = "%s  %s  (YOU)" % [code, String(p["name"]).to_upper()]
	var ini: Dictionary = p["initiative"]
	var lines := [
		"Government  %s" % p["government"],
		"GDP %.0f   growth %+.1f%%" % [float(p["gdp"]), float(p["growth"]) * 100.0],
		"Debt %.0f%% GDP   deficit %.0f%%" % [float(p["debt_ratio"]) * 100.0, float(p["deficit"]) * 100.0],
		"Reserves    %.1f" % float(p["reserves"]),
		"",
		"Stability   %.0f" % float(p["stability"]),
		"  prosperity %.0f  security %.0f" % [float(p["prosperity"]), float(p["security"])],
		"  legitimacy %.0f  war weariness %.0f" % [float(p["legitimacy"]), float(p["war_weariness"])],
		"Initiative  %d left of %d (%d banked)" % [int(ini["left"]), int(ini["available"]), int(ini["banked"])],
		"",
		"Military %.0f   arsenal %d   %s" % [float(p["military"]), int(p["arsenal"]), p["mobilization"]],
		"Energy      %s, net exports %.1f" % [p["energy_policy"], float(p["energy_net_exports"])],
	]
	if p["crisis_pending"]:
		lines.append("")
		lines.append("TRANSITION CRISIS OPEN since turn %s: reform or crack down." % S.opt(p["crisis_since"]))
	var ts := []
	for t in p["treaties"]:
		ts.append("%s %s" % [t["kind"], t["with"]])
	lines.append("")
	lines.append("Treaties: " + (", ".join(ts) if ts.size() > 0 else "none"))
	lines.append("Sanctioning: " + _codes(p["sanctioning"]))
	lines.append("Sanctioned by: " + _codes(p["sanctioned_by"]))
	body.text = "\n".join(lines)


func _foreign(f: Dictionary) -> void:
	if f.is_empty():
		title.text = code
		body.text = "No file on this government."
		return
	title.text = "%s  %s" % [code, String(f["name"]).to_upper()]
	var lines := [
		"Government  %s (%s)" % [f["government"], f["alignment"]],
		"Coverage    %.0f   (estimates: value [low-high])" % float(f["coverage"]),
		"GDP         %.0f" % float(f["gdp"]),
		"Debt/GDP    %s" % S.est(_pct(f["debt_ratio"])),
		"Stability   %s" % S.opt(f["stability_band"]),
		"",
		"Military    %s" % S.est(f["military"]),
		"  land %s" % S.est(f["land"]),
		"  naval %s" % S.est(f["naval"]),
		"  air %s" % S.est(f["air"]),
		"Arsenal (visible) %d   programme %s" % [int(f["arsenal"]), S.opt(f["programme"])],
		"Arms industry %.0f   tech %.0f" % [float(f["arms_industry"]), float(f["military_tech"])],
		"Energy      %s, net exports %.1f" % [f["energy_policy"], float(f["energy_net_exports"])],
		"Budget      %s" % _budget(f["budget"]),
		"",
		"Their opinion of us %+.0f   ours of them %+.0f" % [float(f["their_opinion_of_us"]), float(f["our_opinion_of_them"])],
		"Tension %.0f   trust %.0f" % [float(f["tension"]), float(f["trust"])],
		"Credibility: back %.0f  threat %.0f  norm %.0f" % [float(f["credibility_back"]), float(f["credibility_threat"]), float(f["credibility_norm"])],
		"",
		"Treaties with us: " + _codes(f["treaties_with_us"]),
		"We sanction: %s   they sanction us: %s" % [_yn(f["we_sanction"]), _yn(f["sanctions_us"])],
		"At war: %s   with us: %s" % [_yn(f["at_war"]), _yn(f["at_war_with_us"])],
	]
	body.text = "\n".join(lines)


func _pct(x: Variant) -> Variant:
	if x == null:
		return null
	return [float(x[0]) * 100.0, float(x[1]) * 100.0, float(x[2]) * 100.0]


func _budget(b: Variant) -> String:
	if b == null:
		return "unknown"
	return "mil %.0f%% dev %.0f%% wel %.0f%% int %.0f%%" % [
		float(b["military"]) * 100.0, float(b["development"]) * 100.0,
		float(b["welfare"]) * 100.0, float(b["intelligence"]) * 100.0]


func _codes(xs: Array) -> String:
	if xs.is_empty():
		return "none"
	var out := []
	for x in xs:
		out.append(str(x))
	return ", ".join(out)


func _yn(b: Variant) -> String:
	return "YES" if bool(b) else "no"
