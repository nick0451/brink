// CRT pass (V-1): scanlines, slight barrel curve, phosphor vignette.
shader_type canvas_item;

uniform sampler2D screen_tex : hint_screen_texture, filter_linear;
uniform float curve = 0.06;
uniform float scan = 0.18;

void fragment() {
	vec2 uv = SCREEN_UV * 2.0 - 1.0;
	uv *= 1.0 + curve * dot(uv, uv) * 0.25;
	uv = uv * 0.5 + 0.5;
	if (uv.x < 0.0 || uv.x > 1.0 || uv.y < 0.0 || uv.y > 1.0) {
		COLOR = vec4(0.0, 0.0, 0.0, 1.0);
	} else {
		vec3 c = texture(screen_tex, uv).rgb;
		float line = 0.5 + 0.5 * sin(FRAGCOORD.y * 3.14159);
		c *= 1.0 - scan * (1.0 - line);
		vec2 v = SCREEN_UV - 0.5;
		c *= 1.0 - dot(v, v) * 0.9;
		COLOR = vec4(c, 1.0);
	}
}
