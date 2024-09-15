#version 140

in vec3 position;
in vec3 color_multiplier;

uniform mat4 world_transform;

out vec3 color_multiplier_computed;

void main() {
    gl_Position = world_transform * vec4(position, 1.0);
    color_multiplier_computed = color_multiplier;
}
