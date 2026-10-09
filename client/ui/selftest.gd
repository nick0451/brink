extends Node
## Headless check (`-- --selftest` plus `--player=USA` or `--spectate`):
## drives the real panels through a few turns and prints what they show,
## then quits. Run:
##   godot --headless --path client -- --player=USA --selftest

var main: Node


func _say(x: Variant) -> void:
	print("SELFTEST ", x)


func run() -> void:
	await get_tree().process_frame
	var sim: Node = main.sim
	if not main.started and main.start_ui != null:
		# No mode argument: exercise the start screen (pick SOV by its button).
		for b in main.start_ui.find_children("*", "Button", true, false):
			if b is Button:
				_say("start button: %s%s" % [b.text, " (disabled)" if b.disabled else ""])
		for x in OS.get_cmdline_user_args():
			if x.begins_with("--startshot="):
				for i in 5:
					await get_tree().process_frame
				get_viewport().get_texture().get_image().save_png(x.substr(12))
		main.start_ui._pick("CHN")  # not selectable: must be refused
		_say("start error: " + main.start_ui.error_label.text)
		main.start_ui._pick("SOV")
	_say("mode=%s started=%s" % [main.player if main.player != "" else "spectate", main.started])
	_say("playable=%s" % [sim.playable()])
	if main.player != "":
		await _player_turns(sim)
	else:
		for i in 3:
			main._next_turn()
			_say("turn %d  wars=%d" % [i + 1, sim.wars().size()])
		_say("hud: " + main.hud.text.replace("\n", " | "))
	var probe: String = "SOV" if main.player != "SOV" else "USA"
	var at: Vector2 = main.camera.unproject_position(main.capitals[probe])
	_say("click at capital of %s %s -> picked %s" % [probe, at, main._pick(at)])
	_say("country panel: " + main.country_ui.title.text + " | " + " / ".join(main.country_ui.body.text.split("\n").slice(0, 8)))
	for a in OS.get_cmdline_user_args():
		if a.begins_with("--shot="):  # windowed runs only: save the screen
			for i in 10:
				await get_tree().process_frame
			get_viewport().get_texture().get_image().save_png(a.substr(7))
			_say("screenshot " + a.substr(7))
	_say("done")
	get_tree().quit()


func _player_turns(sim: Node) -> void:
	var o: Node = main.orders_ui
	var me: Dictionary = sim.player_state()
	_say("own: stab %.1f  gdp %.0f  init %s  budget_target %s" % [float(me["stability"]), float(me["gdp"]), me["initiative"], me["budget_target"]])
	var adv: Array = sim.advice()
	_say("advice (%d):" % adv.size())
	for a in adv.slice(0, 5):
		_say("   %s  cost %d  spec %s  top reason %s" % [a["text"], int(a["cost"]), a["spec"], a["reasons"][0] if a["reasons"].size() > 0 else "-"])
	var bt: Dictionary = me["budget_target"]
	var r: Dictionary = o.queue({"kind": "budget", "military": float(bt["military"]) - 0.05,
		"development": float(bt["development"]) + 0.05, "welfare": bt["welfare"], "intelligence": bt["intelligence"]})
	_say("queue budget -> %s" % [r])
	for spec in [
		{"kind": "sanction", "target": "IRN"},       # USA: already in force
		{"kind": "sanction", "target": "SYR"},
		{"kind": "denounce", "target": "CUB"},
		{"kind": "declare_war", "target": "AFG", "aim": "Punitive"},  # 2: over budget
		{"kind": "deficit", "amount": 0.08},
	]:
		r = o.queue(spec)
		_say("queue %s -> %s" % [spec, r])
	_say("pending: %s" % [sim.pending()])
	await get_tree().process_frame
	_say("orders panel rows: %d, status '%s'" % [o.body.get_child_count(), o.last_msg])
	var f: Dictionary = sim.foreign("SOV" if main.player != "SOV" else "USA")
	_say("foreign: military %s debt %s stab %s budget %s coverage %s opinion %s" % [f.get("military"), f.get("debt_ratio"), f.get("stability_band"), f.get("budget"), f.get("coverage"), f.get("their_opinion_of_us")])
	for i in 3:
		if i == 2:
			main._delegate_turn()
			_say("(turn %d delegated; delegate now %s)" % [i + 1, sim.player_state()["delegate"]])
		else:
			main._next_turn()
		_say("turn %d report:" % (i + 1))
		for line in main.last_report.slice(0, 10):
			_say("   " + line)
		_say("   hud: " + main.hud.text.replace("\n", " | "))
		await get_tree().process_frame
	me = sim.player_state()
	_say("after: stab %.1f  budget %s  sanctioning %s" % [float(me["stability"]), me["budget"], me["sanctioning"]])
	# Game-over screen renders (forced, for the check only).
	main._show_game_over("Coup")
	_say("game over screen: " + main.over_ui.get_child(0).get_child(1).text)
	main.over_ui.queue_free()
