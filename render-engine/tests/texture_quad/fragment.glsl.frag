#version 460

layout(location = 0) in vec2 uv;

layout(location = 0) out vec4 f_color;

layout(set = 0, binding = 0) uniform sampler2D color_tex;

void main() {
	f_color = vec4(texture(color_tex, uv));
}