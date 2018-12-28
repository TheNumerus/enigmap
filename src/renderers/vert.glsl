#version 140

uniform float total_x;
uniform float total_y;
uniform float tile_x;
uniform float tile_y;
uniform float mult;
uniform float win_size;

in vec3 position;
in vec2 world_position;
in vec3 color;

out vec3 v_color;

void main() {
    float x = ((position.x + world_position.x) * 2)/(win_size / mult) - 1 - 2 * tile_x;
    float y = ((position.y + world_position.y) * 2)/(win_size / mult) - 1 - 2 * tile_y;
    gl_Position = vec4(x, y, position.z, 1.0);
    v_color = color;
}