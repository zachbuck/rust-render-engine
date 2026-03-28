#version 460

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec2 uv;

layout(set = 0, binding = 0) uniform UBO {
	mat4 transform;
};

void main() {
	gl_Position = transform * vec4(position, 1.0);
}