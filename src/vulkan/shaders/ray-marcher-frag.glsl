#version 460

layout(location = 0) out vec4 f_color;

layout(push_constant) uniform constants {
	vec2 windowSize;
	float fov;
	float nearPlane;
} PushConstants;

void main() {
	vec2 mappedCoords = 2 * (gl_FragCoord.xy / PushConstants.windowSize) - 1;

	f_color = vec4(vec3(length(mappedCoords)), 1.0);
}
