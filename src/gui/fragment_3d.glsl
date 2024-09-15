#version 140

in vec3 color_multiplier_computed;
out vec4 color;

void main() {
    color = vec4(color_multiplier_computed, 1.0);
}
