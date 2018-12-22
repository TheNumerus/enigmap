#version 140

in vec3 v_color;

out vec4 color;

void main() {
    color = vec4(v_color.x, v_color.y, v_color.z, 1.0);
}