#version 140

uniform sampler2D tex;

in float v_color_diff;
in vec2 v_tex_coords;

out vec4 color;

void main() {
    vec4 tex = texture(tex, v_tex_coords);
    if(tex.a < 0.5) {
        discard;
    }
    color = vec4(tex.rgb * v_color_diff, 1.0);
}