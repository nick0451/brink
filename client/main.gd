extends Node3D
## BRINK V-1 visual spike. Builds the whole scene in code:
## hologram vector map, orbit camera with three zoom bands (world / area /
## city), event markers, NES narration popups, CRT post pass.
## Presentation only: everything drawn comes from BrinkSim (the Rust sim).
##
## Controls: Space = next turn, P = autoplay, wheel = zoom,
## right/middle drag or WASD = pan. User arg `--demo` runs a scripted tour.

const DEG := 0.1                      # world units per degree of lon/lat
const HOLO := Color(0.25, 1.0, 0.65)  # phosphor green
const BAND_AREA := 22.0               # camera distance thresholds
const BAND_CITY := 5.0
var seed := 7

var sim: Node
var pivot: Node3D
var camera: Camera3D
var distance := 48.0
var target_distance := 48.0

var capitals := {}        # code -> Vector3
var city_sprites := []    # [{code, sprite, light}]
var city_labels := []     # Label3D, area band
var nation_dots := {}     # code -> MeshInstance3D
var markers := []         # [{node, age}]
var tension_mesh: MeshInstance3D
var damage := {}          # code -> 0..100, persists and decays
var nuked := {}           # code -> true, permanent

var textures := {}
var popup_queue := []
var popup: PanelContainer
var popup_portrait: TextureRect
var popup_speaker: Label
var popup_text: Label
var popup_time := 0.0
var hud: Label
var autoplay := false
var turn_clock := 0.0
var demo := false
var demo_t := 0.0
var demo_focus := Vector3.ZERO
var mono: SystemFont


func _ready() -> void:
	for a in OS.get_cmdline_user_args():
		if a == "--demo":
			demo = true
		elif a.begins_with("--seed="):
			seed = int(a.substr(7))
	mono = SystemFont.new()
	mono.font_names = PackedStringArray(["Lucida Console", "Consolas", "Courier New"])
	mono.antialiasing = TextServer.FONT_ANTIALIASING_NONE
	_build_environment()
	_build_camera()
	_build_map()
	_build_ui()
	sim = ClassDB.instantiate("BrinkSim")
	add_child(sim)
	var root := ProjectSettings.globalize_path("res://").path_join("..")
	if not sim.load(root.path_join("data/scenarios/1980.ron"), root.path_join("data/voice/lines.ron"), seed):
		push_error("BRINK: scenario failed to load")
		return
	_place_nations()
	_refresh()
	if demo:
		_demo_start()


# ---------------------------------------------------------------- scene

func _build_environment() -> void:
	var env := Environment.new()
	env.background_mode = Environment.BG_COLOR
	env.background_color = Color(0.01, 0.03, 0.03)
	env.glow_enabled = true
	env.glow_intensity = 1.2
	env.glow_bloom = 0.15
	env.glow_hdr_threshold = 0.8
	env.glow_blend_mode = Environment.GLOW_BLEND_MODE_ADDITIVE
	env.set_glow_level(0, 1.0)
	env.set_glow_level(2, 1.0)
	env.set_glow_level(4, 0.6)
	var we := WorldEnvironment.new()
	we.environment = env
	add_child(we)


func _build_camera() -> void:
	pivot = Node3D.new()
	add_child(pivot)
	camera = Camera3D.new()
	camera.fov = 40.0
	camera.far = 500.0
	pivot.add_child(camera)
	pivot.position = _project(20.0, 30.0)


func _project(lon: float, lat: float) -> Vector3:
	return Vector3(lon * DEG, 0.0, -lat * DEG)


func _glow(color: Color, energy: float) -> StandardMaterial3D:
	var m := StandardMaterial3D.new()
	m.shading_mode = BaseMaterial3D.SHADING_MODE_UNSHADED
	m.albedo_color = color * energy  # HDR colour > 1 feeds the glow pass
	m.vertex_color_use_as_albedo = true
	return m


func _line_mesh(polylines: Array, color: Color, energy: float, y := 0.0) -> MeshInstance3D:
	var verts := PackedVector3Array()
	var cols := PackedColorArray()
	for line in polylines:
		for k in range(line.size() - 1):
			var a: Array = line[k]
			var b: Array = line[k + 1]
			if absf(a[0] - b[0]) > 180.0:
				continue  # dateline wrap
			verts.append(_project(a[0], a[1]) + Vector3(0, y, 0))
			verts.append(_project(b[0], b[1]) + Vector3(0, y, 0))
			cols.append(Color.WHITE)
			cols.append(Color.WHITE)
	var arrays := []
	arrays.resize(Mesh.ARRAY_MAX)
	arrays[Mesh.ARRAY_VERTEX] = verts
	arrays[Mesh.ARRAY_COLOR] = cols
	var mesh := ArrayMesh.new()
	mesh.add_surface_from_arrays(Mesh.PRIMITIVE_LINES, arrays)
	var mi := MeshInstance3D.new()
	mi.mesh = mesh
	mi.material_override = _glow(color, energy)
	add_child(mi)
	return mi


func _build_map() -> void:
	# Graticule every 10 degrees, dim.
	var grid := []
	for lon in range(-180, 181, 10):
		grid.append([[lon, -60], [lon, 80]])
	for lat in range(-60, 81, 10):
		grid.append([[-180, lat], [180, lat]])
	_line_mesh(grid, Color(0.1, 0.35, 0.3), 0.6, -0.01)
	var data: Dictionary = JSON.parse_string(FileAccess.get_file_as_string("res://data/map.json"))
	_line_mesh(data["coastline"], HOLO, 2.2)
	_line_mesh(data["borders"], Color(0.2, 0.7, 0.9), 0.9)
	var city_tex := {}
	for s in ["intact", "damaged", "burning"]:
		city_tex[s] = _tex("city/%s.png" % s)
	for c in data["cities"]:
		var p := _project(c["lon"], c["lat"])
		if c["capital"]:
			capitals[c["country"]] = p
		var lab := Label3D.new()
		lab.text = String(c["name"]).to_upper()
		lab.font = mono
		lab.font_size = 32
		lab.pixel_size = 0.0006
		lab.fixed_size = true  # constant on-screen size in every band
		lab.modulate = HOLO * 1.4
		lab.billboard = BaseMaterial3D.BILLBOARD_ENABLED
		lab.position = p + Vector3(0, 0.25, 0.12)
		lab.no_depth_test = true
		add_child(lab)
		city_labels.append(lab)
		var spr := Sprite3D.new()
		spr.texture = city_tex["intact"]
		spr.texture_filter = BaseMaterial3D.TEXTURE_FILTER_NEAREST
		spr.axis = Vector3.AXIS_Y
		spr.pixel_size = 0.004 * (0.6 + 0.3 * c["size"])
		spr.position = p + Vector3(0, 0.005, 0)
		add_child(spr)
		var light := OmniLight3D.new()
		light.light_color = Color(1.0, 0.45, 0.1)
		light.omni_range = 0.6
		light.light_energy = 0.0
		light.position = p + Vector3(0, 0.15, 0)
		add_child(light)
		city_sprites.append({"code": c["country"], "sprite": spr, "light": light, "tex": city_tex})
	tension_mesh = MeshInstance3D.new()
	tension_mesh.material_override = _glow(Color.WHITE, 1.0)
	add_child(tension_mesh)


func _tex(rel: String) -> Texture2D:
	if textures.has(rel):
		return textures[rel]
	var path := ProjectSettings.globalize_path("res://art/" + rel)
	var t: Texture2D = null
	if FileAccess.file_exists(path):
		t = ImageTexture.create_from_image(Image.load_from_file(path))
	textures[rel] = t
	return t


func _place_nations() -> void:
	for c in sim.countries():
		var code: String = c["code"]
		if not capitals.has(code):
			continue
		var dot := MeshInstance3D.new()
		var sphere := SphereMesh.new()
		sphere.radius = 0.1
		sphere.height = 0.2
		dot.mesh = sphere
		dot.position = capitals[code] + Vector3(0, 0.05, 0)
		add_child(dot)
		nation_dots[code] = dot


# ---------------------------------------------------------------- UI

func _nes_box() -> StyleBoxFlat:
	var sb := StyleBoxFlat.new()
	sb.bg_color = Color(0, 0, 0, 0.92)
	sb.border_color = Color(0.95, 0.95, 0.95)
	sb.set_border_width_all(4)
	sb.set_content_margin_all(12)
	return sb


func _build_ui() -> void:
	var ui := CanvasLayer.new()
	ui.layer = 5
	add_child(ui)
	hud = Label.new()
	hud.add_theme_font_override("font", mono)
	hud.add_theme_font_size_override("font_size", 18)
	hud.add_theme_color_override("font_color", HOLO)
	hud.position = Vector2(16, 12)
	ui.add_child(hud)

	popup = PanelContainer.new()
	popup.add_theme_stylebox_override("panel", _nes_box())
	popup.position = Vector2(24, 520)
	popup.custom_minimum_size = Vector2(760, 150)
	popup.visible = false
	ui.add_child(popup)
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 16)
	popup.add_child(row)
	popup_portrait = TextureRect.new()
	popup_portrait.custom_minimum_size = Vector2(112, 112)
	popup_portrait.texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
	popup_portrait.expand_mode = TextureRect.EXPAND_IGNORE_SIZE
	popup_portrait.stretch_mode = TextureRect.STRETCH_KEEP_ASPECT_CENTERED
	row.add_child(popup_portrait)
	var col := VBoxContainer.new()
	col.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	row.add_child(col)
	popup_speaker = Label.new()
	popup_speaker.add_theme_font_override("font", mono)
	popup_speaker.add_theme_font_size_override("font_size", 16)
	popup_speaker.add_theme_color_override("font_color", Color(1.0, 0.8, 0.2))
	col.add_child(popup_speaker)
	popup_text = Label.new()
	popup_text.add_theme_font_override("font", mono)
	popup_text.add_theme_font_size_override("font_size", 17)
	popup_text.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	popup_text.custom_minimum_size = Vector2(600, 0)
	col.add_child(popup_text)

	var crt_layer := CanvasLayer.new()
	crt_layer.layer = 10
	add_child(crt_layer)
	var crt := ColorRect.new()
	crt.set_anchors_preset(Control.PRESET_FULL_RECT)
	crt.mouse_filter = Control.MOUSE_FILTER_IGNORE
	var mat := ShaderMaterial.new()
	mat.shader = load("res://crt.gdshader")
	crt.material = mat
	crt_layer.add_child(crt)


# ---------------------------------------------------------------- turns

func _next_turn() -> void:
	var out: Dictionary = sim.step()
	for e in out["events"]:
		_add_marker(e)
		if e["kind"] == "nuclear":
			nuked[e["b"]] = true
	for n in out["narration"]:
		if int(n["gravity"]) >= 2 or String(n["text"]) != "":
			popup_queue.append(n)
	if popup_queue.size() > 6:  # keep the newest; the ledger keeps the rest
		popup_queue = popup_queue.slice(popup_queue.size() - 6)
	_refresh()


func _refresh() -> void:
	var gt: float = sim.global_tension()
	var defcon := clampi(5 - int(gt / 20.0), 1, 5)
	hud.text = "BRINK  %.1f   TURN %d   DEFCON %d   OIL x%.2f   RATES %.1f%%   WARS %d" % [
		sim.year(), int((sim.year() - 1980.0) * 4.0), defcon, sim.energy_price(), sim.interest_rate(), sim.wars().size()]
	var countries: Array = sim.countries()
	for c in countries:
		var dot: MeshInstance3D = nation_dots.get(c["code"])
		if dot == null:
			continue
		var col := Color(0.9, 0.7, 0.2)
		match String(c["alignment"]):
			"west": col = Color(0.3, 0.6, 1.0)
			"east": col = Color(1.0, 0.25, 0.2)
		if c["at_war"]:
			col = Color(1.0, 0.1, 0.05)
		dot.material_override = _glow(col, 2.5 if c["at_war"] else 1.6)
		var s := 0.6 + sqrt(maxf(float(c["power"]), 0.0)) * 0.08
		dot.scale = Vector3.ONE * s
	# Damage: losing side of each war; decays slowly in peace (D57: no people).
	var hit := {}
	for w in sim.wars():
		var p: float = w["progress"]
		hit[w["defender"]] = maxf(hit.get(w["defender"], 0.0), p)
		hit[w["attacker"]] = maxf(hit.get(w["attacker"], 0.0), -p)
	for code in damage.keys():
		damage[code] = maxf(damage[code] - 2.0, 0.0)
	for code in hit:
		damage[code] = maxf(damage.get(code, 0.0), hit[code])
	for c in city_sprites:
		var lvl: float = damage.get(c["code"], 0.0)
		var state := "intact"
		if nuked.has(c["code"]) or lvl > 45.0:
			state = "burning"
		elif lvl > 15.0:
			state = "damaged"
		c["sprite"].texture = c["tex"][state]
		c["light"].light_energy = 2.0 if state == "burning" else 0.0
	_draw_tension(countries)
	for m in markers:
		m["age"] += 1


func _draw_tension(countries: Array) -> void:
	var im := ImmediateMesh.new()
	var codes := []
	for c in countries:
		if capitals.has(c["code"]):
			codes.append(c["code"])
	var any := false
	for i in codes.size():
		for j in range(i + 1, codes.size()):
			var t: float = sim.tension(codes[i], codes[j])
			if t < 55.0:
				continue
			if not any:
				im.surface_begin(Mesh.PRIMITIVE_LINES)
				any = true
			var col := Color(1.0, 0.85, 0.2).lerp(Color(1.0, 0.1, 0.05), clampf((t - 55.0) / 35.0, 0.0, 1.0)) * 2.0
			var a: Vector3 = capitals[codes[i]]
			var b: Vector3 = capitals[codes[j]]
			var prev := a
			for k in range(1, 17):  # arc above the map
				var f := k / 16.0
				var p := a.lerp(b, f) + Vector3(0, sin(f * PI) * a.distance_to(b) * 0.18, 0)
				im.surface_set_color(col)
				im.surface_add_vertex(prev)
				im.surface_set_color(col)
				im.surface_add_vertex(p)
				prev = p
	if any:
		im.surface_end()
	tension_mesh.mesh = im


func _add_marker(e: Dictionary) -> void:
	var at: String = e["b"] if e["kind"] in ["war", "nuclear", "sanction", "joined_war"] and e["b"] != "" else e["a"]
	if not capitals.has(at):
		return
	var tex := _tex("markers/%s.png" % e["kind"])
	if tex == null:
		return
	for m in markers:  # one fresh marker per kind per capital
		if m["at"] == at and m["kind"] == e["kind"]:
			m["age"] = 0
			return
	var spr := Sprite3D.new()
	spr.texture = tex
	spr.texture_filter = BaseMaterial3D.TEXTURE_FILTER_NEAREST
	spr.billboard = BaseMaterial3D.BILLBOARD_ENABLED
	spr.no_depth_test = true
	spr.fixed_size = true
	spr.pixel_size = 0.0016
	spr.position = capitals[at] + Vector3(randf_range(-0.3, 0.3), 0.4, randf_range(-0.2, 0.2))
	add_child(spr)
	markers.append({"node": spr, "age": 0, "at": at, "kind": e["kind"]})


# ---------------------------------------------------------------- frame

func _process(delta: float) -> void:
	if demo:
		_demo_update(delta)
	elif autoplay:
		turn_clock += delta
		if turn_clock > 1.5:
			turn_clock = 0.0
			_next_turn()
	_pan_keys(delta)
	distance = lerpf(distance, target_distance, clampf(delta * 6.0, 0.0, 1.0))
	# Near top-down far out, tilting toward the horizon as you close in.
	var pitch := deg_to_rad(lerpf(-58.0, -82.0, clampf((distance - BAND_CITY) / 40.0, 0.0, 1.0)))
	camera.position = Vector3(0, -sin(pitch), cos(pitch)) * distance
	camera.look_at(pivot.global_position)
	_apply_bands()
	_update_markers(delta)
	_update_popup(delta)


func _apply_bands() -> void:
	var area := distance < BAND_AREA
	var city := distance < BAND_CITY
	for lab in city_labels:
		lab.visible = area
	for c in city_sprites:
		c["sprite"].visible = area
		c["sprite"].modulate = Color(1, 1, 1, 1) if city else Color(1, 1, 1, 0.85)
	for code in nation_dots:
		nation_dots[code].visible = not city
	tension_mesh.visible = not city


func _update_markers(delta: float) -> void:
	var keep := []
	for m in markers:
		var node: Sprite3D = m["node"]
		var life: int = m["age"]
		if life > 6:
			node.queue_free()
			continue
		node.modulate.a = 1.0 - life / 7.0
		node.visible = distance > BAND_CITY  # the city view shows the city itself
		keep.append(m)
	markers = keep


func _update_popup(delta: float) -> void:
	if popup.visible:
		popup_time += delta
		popup_text.visible_characters = int(popup_time * 45.0)
		if popup_time > 4.5:
			popup.visible = false
	elif not popup_queue.is_empty():
		_show_popup(popup_queue.pop_front())


func _show_popup(n: Dictionary) -> void:
	var g: int = n["gravity"]
	var key: String = n["portrait"]
	var tex: Texture2D = null
	if key == "seal":
		tex = _tex("portraits/seal.png")
	elif key != "":
		tex = _tex("portraits/%s.png" % key)
		if tex == null:
			var generic := "generic_revolutionary" if key.ends_with("revolutionary") else "generic_base"
			tex = _tex("portraits/%s.png" % generic)
	popup_portrait.texture = tex
	popup_portrait.visible = tex != null
	var fact: String = n["fact"]
	if g >= 4 or String(n["text"]) == "":
		# Gravity gate: the plain fact, nothing else.
		popup_speaker.text = "LEDGER"
		popup_text.text = fact.to_upper()
	else:
		popup_speaker.text = "%s — %s" % [String(n["speaker"]).to_upper(), String(n["heading"]).to_upper()]
		popup_text.text = "%s\n\n%s" % [fact, n["text"]]
	popup_text.visible_characters = 0
	popup_time = 0.0
	popup.visible = true


# ---------------------------------------------------------------- input

func _pan_keys(delta: float) -> void:
	var v := Vector3.ZERO
	if Input.is_physical_key_pressed(KEY_W): v.z -= 1
	if Input.is_physical_key_pressed(KEY_S): v.z += 1
	if Input.is_physical_key_pressed(KEY_A): v.x -= 1
	if Input.is_physical_key_pressed(KEY_D): v.x += 1
	pivot.position += v * distance * 0.8 * delta


func _unhandled_input(event: InputEvent) -> void:
	if event is InputEventMouseButton and event.pressed:
		if event.button_index == MOUSE_BUTTON_WHEEL_UP:
			target_distance = maxf(target_distance * 0.85, 1.2)
		elif event.button_index == MOUSE_BUTTON_WHEEL_DOWN:
			target_distance = minf(target_distance / 0.85, 70.0)
	elif event is InputEventMouseMotion and (event.button_mask & (MOUSE_BUTTON_MASK_RIGHT | MOUSE_BUTTON_MASK_MIDDLE)):
		pivot.position -= Vector3(event.relative.x, 0, event.relative.y) * distance * 0.0012
	elif event is InputEventKey and event.pressed and not event.echo:
		if event.keycode == KEY_SPACE:
			_next_turn()
		elif event.keycode == KEY_P:
			autoplay = not autoplay


# ---------------------------------------------------------------- demo tour

func _demo_start() -> void:
	# Fast-forward until a war is running, then tour its losing capital.
	for i in 80:
		_next_turn()
		var going := false
		for w in sim.wars():
			going = going or absf(w["progress"]) > 30.0
		if going:
			break
	popup_queue.clear()
	var focus := "IRQ"
	var worst := -1.0
	for w in sim.wars():
		var loser: String = w["defender"] if w["progress"] >= 0.0 else w["attacker"]
		if absf(w["progress"]) > worst and capitals.has(loser):
			worst = absf(w["progress"])
			focus = loser
	demo_focus = capitals.get(focus, Vector3.ZERO)
	pivot.position = Vector3(demo_focus.x * 0.3, 0, demo_focus.z * 0.3)
	distance = 55.0
	target_distance = 55.0
	print("BRINK demo: focus ", focus, " wars ", sim.wars())


func _demo_update(delta: float) -> void:
	demo_t += delta
	turn_clock += delta
	if turn_clock > 1.0:
		turn_clock = 0.0
		_next_turn()
	# 0-3 s world, 3-6 s glide to area, 6-9 s down to city.
	var f := clampf((demo_t - 2.0) / 3.0, 0.0, 1.0)
	pivot.position = pivot.position.lerp(demo_focus, clampf(delta * 1.5 * f, 0.0, 1.0))
	if demo_t > 2.0:
		target_distance = 12.0
	if demo_t > 6.0:
		target_distance = 2.2
