#version 410 core

in vec2 a_Pos;
in vec3 a_Color;
out vec4 v_Color;

uniform mat4 modelViewProjMatrix;

void main() {
    v_Color = vec4(a_Color, 1.0);
    gl_Position = modelViewProjMatrix * vec4(a_Pos, 0.0, 1.0);
}
