#version 140

uniform mat4x4 transform;

in vec3 position;
in vec2 world_position;
in float color_diff;
in vec2 tex_coords;
in mat2x2 rotation;

out vec2 v_tex_coords;
out float v_color_diff;

void main() {
    float half_ratio = 1.1547 / 2.0;
    vec2 rot_center = vec2(0.5, half_ratio);
    // center hex, rotate it and put it back in
    vec2 xy = (position.xy - rot_center) * rotation + rot_center;
    gl_Position = vec4(xy + world_position, position.z, 1.0) * transform;
    v_color_diff = color_diff;
    v_tex_coords = tex_coords;
}