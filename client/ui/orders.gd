extends PanelContainer
## Orders panel (V-2a). Every control builds an order spec (API.md) and
## queues it with `queue_order(spec)`; the bridge dry-runs the whole queue
## and answers with the cost, the Initiative left and the simulation's own
## refusal reason. Also: the pending queue (with remove), the cabinet's
## advice (toggle, on by default; D105 #2), End turn and Delegate.
## The panel is rebuilt after every change; slider drafts survive rebuilds
## until the turn ends.

signal end_turn
signal delegate_turn

const S := preload("res://ui/style.gd")

const BUDGET_KEYS := ["military", "development", "welfare", "intelligence"]
const MOBILIZATION := ["Peacetime", "Partial", "Full", "Total"]
const ENERGY := ["Restrain", "Normal", "Flood"]
const STANCE := ["Tight", "Neutral", "Loose"]
## [label, spec template]; the target is added at queue time.
const ACTIONS := [
	["Propose trade pact", {"kind": "propose_treaty", "treaty": "Trade"}],
	["Propose non-aggression", {"kind": "propose_treaty", "treaty": "NonAggression"}],
	["Propose defensive alliance", {"kind": "propose_treaty", "treaty": "DefensiveAlliance"}],
	["Guarantee", {"kind": "guarantee"}],
	["Sanction", {"kind": "sanction"}],
	["Lift sanction", {"kind": "lift_sanction"}],
	["Denounce", {"kind": "denounce"}],
	["Send aid (% of our GDP)", {"kind": "aid"}],
	["Declare war: punitive", {"kind": "declare_war", "aim": "Punitive"}],
	["Declare war: limited", {"kind": "declare_war", "aim": "Limited"}],
	["Declare war: major", {"kind": "declare_war", "aim": "Major"}],
]

var sim: Node
var player := ""
var advisor_on := true
var target := ""
var action := 0
var aid_pct := 0.5
var draft := {}            # slider drafts: budget keys + "deficit"
var last_msg := ""
var last_ok := true
var scroll: ScrollContainer
var body: VBoxContainer
var target_opt: OptionButton
var targets: Array = []


func setup(sim_node: Node, player_code: String) -> void:
	sim = sim_node
	player = player_code
	theme = S.theme()
	if custom_minimum_size.x == 0.0:
		custom_minimum_size = Vector2(420, 600)
	scroll = ScrollContainer.new()
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	add_child(scroll)
	body = VBoxContainer.new()
	body.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	body.add_theme_constant_override("separation", 5)
	scroll.add_child(body)
	refresh()


## Called by main after every turn: clears drafts and the last message.
func new_turn() -> void:
	draft.clear()
	last_msg = ""
	refresh()


func set_target(code: String) -> void:
	if code != player:
		target = code
		refresh()


## Queue one order spec; records the bridge's answer for the status line.
func queue(spec: Dictionary) -> Dictionary:
	var r: Dictionary = sim.queue_order(spec)
	if r["ok"]:
		last_ok = true
		last_msg = "QUEUED (cost %d, %d Initiative left)" % [int(r["cost"]), int(r["left"])]
	else:
		last_ok = false
		last_msg = "REFUSED: %s" % r["reason"]
	refresh.call_deferred()  # the pressed button is rebuilt; not mid-signal
	return r


func refresh() -> void:
	if sim == null:
		return
	var y := scroll.scroll_vertical
	for c in body.get_children():
		body.remove_child(c)
		c.queue_free()
	var me: Dictionary = sim.player_state()
	if me.is_empty():
		return
	_build_header(me)
	_build_crisis(me)
	_build_proposals()
	if advisor_on:
		_build_advice()
	_build_pending()
	_build_economy(me)
	_build_diplomacy(me)
	_build_treaties(me)
	_build_wars()
	scroll.set_deferred("scroll_vertical", y)


# ------------------------------------------------------------ sections

func _build_header(me: Dictionary) -> void:
	var ini: Dictionary = me["initiative"]
	body.add_child(S.label("%s  ORDERS  —  YEAR %.2f  TURN %d" % [player, float(me["year"]), int(me["turn"])], S.AMBER, 16))
	body.add_child(S.label("INITIATIVE  %d left of %d   (allowance %d, banked %d)" % [
		int(ini["left"]), int(ini["available"]), int(ini["allowance"]), int(ini["banked"])]))
	var adv := CheckButton.new()
	adv.text = "Cabinet advice"
	adv.focus_mode = Control.FOCUS_NONE
	adv.button_pressed = advisor_on
	adv.toggled.connect(func(on: bool) -> void:
		advisor_on = on
		refresh.call_deferred())
	body.add_child(S.row([
		S.button("END TURN [SPACE]", func() -> void: end_turn.emit()),
		S.button("LET THE CABINET RUN THIS TURN", func() -> void: delegate_turn.emit()),
	]))
	body.add_child(adv)
	if last_msg != "":
		body.add_child(S.label(last_msg, S.HOLO if last_ok else S.RED))


func _build_crisis(me: Dictionary) -> void:
	if not me["crisis_pending"]:
		return
	body.add_child(S.header("Transition crisis"))
	body.add_child(S.label("Open since turn %s. Unanswered, it defaults to Crackdown." % S.opt(me["crisis_since"]), S.RED))
	body.add_child(S.row([
		S.button("REFORM", func() -> void: queue({"kind": "reform"})),
		S.button("CRACKDOWN", func() -> void: queue({"kind": "crackdown"})),
	]))


func _build_proposals() -> void:
	var ps: Array = sim.proposals()
	if ps.is_empty():
		return
	body.add_child(S.header("Proposals (lapse this turn)"))
	for p in ps:
		var id: int = p["id"]
		body.add_child(S.label("%s  [accept costs %d]" % [p["text"], int(p["accept_cost"])]))
		body.add_child(S.row([
			S.button("ACCEPT", func() -> void: queue({"kind": "respond", "id": id, "flag": true})),
			S.button("DECLINE", func() -> void: queue({"kind": "respond", "id": id, "flag": false})),
		]))


func _build_advice() -> void:
	var adv: Array = sim.advice()
	body.add_child(S.header("The cabinet recommends"))
	if adv.is_empty():
		body.add_child(S.label("No recommendations this turn.", S.DIM))
		return
	for a in adv:
		var head := "%s  (cost %d)" % [a["text"], int(a["cost"])]
		var rs := []
		for r in a["reasons"]:
			rs.append("  %s %+.1f" % [r["label"], float(r["value"])])
		var lab := S.label(head + ("\n" + "\n".join(rs.slice(0, 4)) if rs.size() > 0 else ""))
		lab.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		var spec: Variant = a["spec"]
		if spec == null:
			lab.text += "\n  (outside your desk: delegate to play it)"
			body.add_child(lab)
		else:
			var s: Dictionary = spec
			var b := S.button("QUEUE", func() -> void: queue(s))
			body.add_child(S.row([lab, b]))


func _build_pending() -> void:
	var pend: Array = sim.pending()
	body.add_child(S.header("Queued orders (%d)" % pend.size()))
	if pend.is_empty():
		body.add_child(S.label("None.", S.DIM))
	for i in pend.size():
		var lab := S.label(String(pend[i]["text"]))
		lab.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		var idx := i
		body.add_child(S.row([lab, S.button("X", func() -> void:
			sim.unqueue(idx)
			last_msg = ""
			refresh.call_deferred())]))


func _slider(key: String, value: float, lo: float, hi: float, step: float, fmt: String) -> HBoxContainer:
	var lab := S.label(key.substr(0, 5).to_upper())
	lab.custom_minimum_size = Vector2(60, 0)
	var sl := HSlider.new()
	sl.min_value = lo
	sl.max_value = hi
	sl.step = step
	sl.value = draft.get(key, value)
	sl.focus_mode = Control.FOCUS_NONE
	sl.custom_minimum_size = Vector2(220, 18)
	sl.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	var val := S.label(fmt % sl.value)
	val.custom_minimum_size = Vector2(60, 0)
	sl.value_changed.connect(func(v: float) -> void:
		draft[key] = v
		val.text = fmt % v)
	return S.row([lab, sl, val])


func _build_economy(me: Dictionary) -> void:
	body.add_child(S.header("Budget (shares, normalised)"))
	var bt: Dictionary = me["budget_target"]
	var bn: Dictionary = me["budget"]
	body.add_child(S.label("now: mil %.0f dev %.0f wel %.0f int %.0f (%%)" % [
		float(bn["military"]) * 100, float(bn["development"]) * 100,
		float(bn["welfare"]) * 100, float(bn["intelligence"]) * 100], S.DIM))
	for k in BUDGET_KEYS:
		body.add_child(_slider(k, float(bt[k]) * 100.0, 0.0, 80.0, 1.0, "%.0f%%"))
	body.add_child(S.button("QUEUE BUDGET", func() -> void:
		var spec := {"kind": "budget"}
		for k in BUDGET_KEYS:
			spec[k] = float(draft.get(k, float(bt[k]) * 100.0)) / 100.0
		queue(spec)))
	body.add_child(_slider("deficit", float(me["deficit"]) * 100.0, 0.0, 30.0, 1.0, "%.0f%%"))
	body.add_child(S.label("Deficit: share of revenue borrowed (debt %.0f%% of GDP)" % (float(me["debt_ratio"]) * 100.0), S.DIM))
	body.add_child(S.button("QUEUE DEFICIT", func() -> void:
		queue({"kind": "deficit", "amount": float(draft.get("deficit", float(me["deficit"]) * 100.0)) / 100.0})))

	body.add_child(S.header("Posture"))
	_level_row("Mobilization", "mobilization", MOBILIZATION, String(me["mobilization"]))
	if float(me["energy_capacity"]) > 0.0:
		_level_row("Energy", "energy_policy", ENERGY, String(me["energy_policy"]))
	if me["reserve_holder"]:
		_level_row("Monetary", "monetary_stance", STANCE, "?")


func _level_row(title: String, kind: String, levels: Array, current: String) -> void:
	var o := S.options(levels, maxi(levels.find(current), 0))
	var lab := S.label("%s (%s)" % [title, current])
	lab.custom_minimum_size = Vector2(170, 0)
	body.add_child(S.row([lab, o, S.button("QUEUE", func() -> void:
		queue({"kind": kind, "level": levels[o.selected]}))]))


func _build_diplomacy(me: Dictionary) -> void:
	body.add_child(S.header("Foreign desk"))
	targets.clear()
	var names := []
	for c in sim.countries():
		if c["code"] != player:
			targets.append(c["code"])
			names.append("%s %s" % [c["code"], c["name"]])
	if target == "" or not targets.has(target):
		target = targets[0] if targets.size() > 0 else ""
	target_opt = S.options(names, targets.find(target))
	target_opt.item_selected.connect(func(i: int) -> void: target = targets[i])
	var labels := []
	for a in ACTIONS:
		labels.append(a[0])
	var act := S.options(labels, action)
	act.item_selected.connect(func(i: int) -> void:
		action = i
		refresh.call_deferred())
	body.add_child(S.row([S.label("Target"), target_opt]))
	body.add_child(S.row([S.label("Action"), act]))
	var spec: Dictionary = ACTIONS[action][1]
	if spec["kind"] == "aid":
		body.add_child(_slider("aid", aid_pct, 0.1, 3.0, 0.1, "%.1f%%"))
	body.add_child(S.button("QUEUE ACTION", func() -> void:
		var s := spec.duplicate()
		s["target"] = target
		if s["kind"] == "aid":
			aid_pct = float(draft.get("aid", aid_pct))
			s["amount"] = float(me["gdp"]) * aid_pct / 100.0
		queue(s)))


func _build_treaties(me: Dictionary) -> void:
	var ts: Array = me["treaties"]
	if ts.is_empty():
		return
	body.add_child(S.header("Our treaties"))
	for t in ts:
		var id: int = t["id"]
		var lab := S.label("%s with %s" % [t["kind"], t["with"]])
		lab.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		body.add_child(S.row([lab, S.button("CANCEL", func() -> void: queue({"kind": "cancel_treaty", "id": id}))]))


func _build_wars() -> void:
	var ws: Array = sim.wars()
	if ws.is_empty():
		return
	body.add_child(S.header("Wars"))
	for w in ws:
		var id: int = w["id"]
		var ours: bool = w["attacker"] == player or w["defender"] == player
		body.add_child(S.label("#%d %s vs %s (%s), progress %+.0f" % [id, w["attacker"], w["defender"], w["aim"], float(w["progress"])]))
		var row: HBoxContainer
		if ours:
			row = S.row([
				S.button("OFFER PEACE", func() -> void: queue({"kind": "offer_peace", "id": id})),
				S.button("LEAVE", func() -> void: queue({"kind": "leave_war", "id": id})),
			])
		else:
			row = S.row([
				S.button("JOIN ATK", func() -> void: queue({"kind": "join_war", "id": id, "side": "Attacker", "band": 3})),
				S.button("JOIN DEF", func() -> void: queue({"kind": "join_war", "id": id, "side": "Defender", "band": 3})),
				S.button("PEACE", func() -> void: queue({"kind": "offer_peace", "id": id})),
				S.button("LEAVE", func() -> void: queue({"kind": "leave_war", "id": id})),
			])
		body.add_child(row)
