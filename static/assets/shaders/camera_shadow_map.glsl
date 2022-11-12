varying vec3 v_normal;
varying vec4 v_light_pos;
varying vec3 v_world_pos;

#ifdef VERTEX_SHADER
attribute vec3 a_v;
attribute vec3 a_vn;
uniform mat4 u_projection_matrix;
uniform mat4 u_view_matrix;
uniform mat4 u_model_matrix;
uniform mat4 u_light_matrix;

mat3 transpose(mat3 mat) {
    return mat3(
        vec3(mat[0].x, mat[1].x, mat[2].x),
        vec3(mat[0].y, mat[1].y, mat[2].y),
        vec3(mat[0].z, mat[1].z, mat[2].z));
}

void main() {
    v_normal = transpose(inverse(mat3(u_model_matrix))) * a_vn;
    v_world_pos = (u_model_matrix * vec4(a_v, 1.0)).xyz;
    v_light_pos = u_light_matrix * vec4(v_world_pos, 1.0);
    gl_Position = u_projection_matrix * u_view_matrix * vec4(v_world_pos, 1.0);
}
#endif

#ifdef FRAGMENT_SHADER
uniform sampler2D u_shadow_map;
uniform ivec2 u_shadow_size;
uniform vec3 u_light_source;

void main() {
    vec3 light_pos = v_light_pos.xyz / v_light_pos.w * 0.5 + 0.5;
    vec3 light_dir = normalize(u_light_source - v_world_pos);
    vec3 normal = normalize(v_normal);
    
    float cos = 1.0; // dot(light_dir, normal); // TODO: fix bias
    float bias = max(0.001, 0.01 * (1.0 - cos));

    vec2 texel_size = 1.0 / vec2(u_shadow_size);
    float shadow = 0.0;
    for (int i = -1; i <= 1; ++i) {
        for (int j = -1; j <= 1; ++j) {
            float pcf_depth = unpack4(texture2D(u_shadow_map, light_pos.xy + vec2(i, j) * texel_size));
            shadow += light_pos.z - bias > pcf_depth ? 1.0 : 0.0;
        }
    }
    shadow /= 9.0;
    if (light_pos.z > 1.0) {
        shadow = 0.0;
    }
    gl_FragColor = vec4(vec3(shadow), 1.0);
}
#endif