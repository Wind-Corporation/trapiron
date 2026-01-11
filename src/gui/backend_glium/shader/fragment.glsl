#version 140

in vec3 color_multiplier_computed;

in vec2 texture_coords_passthru;
uniform sampler2D tex;

out vec4 color;

void main() {
    color = texture(tex, texture_coords_passthru);
    if (color.a != 1.0) {
        discard;
    }
    color *= vec4(color_multiplier_computed, 1.0);
}
