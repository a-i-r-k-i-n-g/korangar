#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec2 texture_coordinates;
layout(location = 3) in int texture_index;

layout(location = 0) out vec3 normal_out;

layout(set = 0, binding = 0) uniform Matrices {
    mat4 view;
    mat4 projection;
} matrices;

layout(push_constant) uniform Constants {
    mat4 world;
} constants;

void main() {
    gl_Position = matrices.projection * matrices.view * constants.world * vec4(position, 1.0);
    normal_out = vec3(0.0, 0.0, 0.0); //vec4(constants.rotation * vec4(normal, 1.0)).xyz;
    //texture_coordinates_out = texture_coordinates;
    //texture_index_out = texture_index;
}
