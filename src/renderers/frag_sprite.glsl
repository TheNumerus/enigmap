#version 140

uniform sampler2D tex;

in float v_color_diff;
in vec2 v_tex_coords;

out vec4 color;

void main() {
    vec4 tex = texture(tex, v_tex_coords);
    color = vec4(pow(tex.r * v_color_diff, 2.2), pow(tex.g * v_color_diff, 2.2), pow(tex.b * v_color_diff, 2.2), 1.0);
}