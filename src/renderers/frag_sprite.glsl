#version 140

uniform sampler2D tex;

in vec3 v_color;
in vec2 v_tex_coords;

out vec4 color;

void main() {
    color = texture(tex, v_tex_coords);
    //color = vec4(pow(v_color.x, 2.2), pow(v_color.y, 2.2), pow(v_color.z, 2.2), 1.0);
}