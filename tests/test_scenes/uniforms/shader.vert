#version 460

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec2 uv;

layout(set = 0, binding = 0) uniform CameraBuffer {
	mat4 camera;
	mat4 projection;
};

layout(set = 1, binding = 0) uniform ObjectBuffer {
	mat4 transform;
};
layout(set = 1, binding = 1) uniform Test {
	vec4 test;
};

void main() {
	gl_Position = camera * projection * transform * vec4(position, 1.0) + test;
}