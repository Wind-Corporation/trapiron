#version 140

in vec3 position;
in vec3 color_multiplier;
in vec2 texture_coords;

uniform mat4x3 world_transform;

out vec3 color_multiplier_computed;
out vec2 texture_coords_passthru;

void main() {
    gl_Position = vec4(world_transform * vec4(position, 1.0), 1.0);
    color_multiplier_computed = color_multiplier;
    texture_coords_passthru = texture_coords;
}
