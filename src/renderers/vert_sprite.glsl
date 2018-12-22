#version 140

uniform float total_x;
uniform float total_y;
uniform float tile_x;
uniform float tile_y;
uniform float mult;
uniform float win_size;

in vec2 position;
in vec2 world_position;
in float color_diff;
in vec2 tex_coords;
in mat2x2 rotation;

out vec2 v_tex_coords;
out float v_color_diff;

void main() {
    float half_ratio = 1.1547 / 2.0;
    vec2 xy = vec2(position.x - 0.5, position.y - half_ratio);
    xy *= rotation;
    float x = ((xy.x + world_position.x + 0.5) * 2)/(win_size / mult) - 1 - 2 * tile_x;
    float y = ((xy.y + world_position.y + half_ratio) * 2)/(win_size / mult) - 1 - 2 * tile_y;
    gl_Position = vec4(x, y, 0.0, 1.0);
    v_color_diff = color_diff;
    v_tex_coords = tex_coords;
}