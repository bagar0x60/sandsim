#version 150 core

in vec3 a_pos;
uniform vec4 a_color;
uniform mat4 u_model_view_proj;
out vec4 v_color;

void main() {
    v_color = a_color;
    gl_Position = u_model_view_proj * vec4(a_pos, 1.0);
}
