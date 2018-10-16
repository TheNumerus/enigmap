#version 140

uniform float total_x;
uniform float total_y;

in vec2 position;
in vec2 world_position;
in vec3 color;

out vec3 v_color;

void main() {
    gl_Position = vec4((position.x + world_position.x)/total_x*2 - 1, (-(position.y + world_position.y)/total_y*2) + 1, 0.0, 1.0);
    v_color = color;
}