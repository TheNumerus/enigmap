#version 140

in vec3 v_color;

out vec4 color;

void main() {
    color = vec4(pow(v_color.x, 2.2), pow(v_color.y, 2.2), pow(v_color.z, 2.2), 1.0);
}