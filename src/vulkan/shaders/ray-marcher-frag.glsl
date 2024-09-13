#version 460

layout(origin_upper_left) in vec4 gl_FragCoord;

layout(location = 0) out vec4 f_color;

layout(push_constant) uniform constants {
	vec2 windowSize;
	float fov;
	float nearPlane;
} PushConstants;

struct Ray {
	vec3 dir;
	vec3 pos;
} ray;

void generateRay() {
	ray.pos.z = PushConstants.nearPlane;

	float halfWidth = tan(radians(PushConstants.fov / 2)) * PushConstants.nearPlane;
	float halfHeight = halfWidth * PushConstants.windowSize.y / PushConstants.windowSize.x;
	float pixelSize = 2 * halfWidth / PushConstants.windowSize.x;

	ray.pos.xy = vec2(1.0, -1.0) * (gl_FragCoord.xy * pixelSize - vec2(halfWidth, halfHeight));

	ray.dir = normalize(ray.pos);
}

const int MAX_ITER = 512;
const float MAX_DIST = 100.0;
const float MIN_DIST = 0.001;

float sdSphere(vec3 p, float r, vec3 transform) {
	return distance(p, transform) - r;
}

float sdScene(vec3 p) {
	return sdSphere(mod(p, 5.0), 0.5, vec3(2.5, 2.5, 2.5));
}

void main() {
	generateRay();

	bool hit = false;
	vec3 origPos = ray.pos;

	for(int i = 0; i < MAX_ITER; i++) {
		float dist = sdScene(ray.pos);
		if(dist > MAX_DIST) {
			break;
		}

		if(dist < MIN_DIST) {
			hit = true;
			break;
		}

		ray.pos = ray.pos + ray.dir * dist;
	}


	if(hit) {
		float totalDist = clamp(distance(origPos, ray.pos), 0.0, MAX_DIST);
		float lightness = 1.0 - totalDist / MAX_DIST;
		f_color = vec4(vec3(lightness), 1.0);
	} else {
		f_color = vec4(0.0, 0.0, 0.0, 1.0);
	}
}
