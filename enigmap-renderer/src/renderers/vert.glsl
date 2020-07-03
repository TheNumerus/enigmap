#version 140

uniform mat4x4 transform;

in vec3 position;
in vec2 world_position;
in vec3 color;

out vec3 v_color;

void main() {
    gl_Position = vec4(position + vec3(world_position, 0.0),1.0) * transform;
    v_color = color;
}