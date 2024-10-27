#version 140

in vec3 position;
in vec3 color_multiplier;
in vec2 texture_coords;

uniform mat4 screen_transform;
uniform mat4x3 view_transform;
uniform mat4x3 world_transform;
uniform vec3 color_multiplier_global;

out vec3 color_multiplier_computed;
out vec2 texture_coords_passthru;

void main() {
    gl_Position = vec4(position, 1.0);
    gl_Position = vec4(world_transform * gl_Position, 1.0);
    gl_Position = vec4(view_transform * gl_Position, 1.0);
    gl_Position = screen_transform * gl_Position;

    color_multiplier_computed = color_multiplier * color_multiplier_global;
    texture_coords_passthru = texture_coords;
}
