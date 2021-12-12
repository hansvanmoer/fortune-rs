#version 330 core

layout (location = 0) in vec3 pos;

uniform mat4 view = mat4(1.0);

void main() {
     gl_Position = view * vec4(pos.x, pos.y, pos.z, 1.0);
}