varying vec2 v_uv;
varying vec3 v_eye_pos;
varying vec3 v_world_pos;
varying vec3 v_normal;
varying vec4 v_light_pos;

#ifdef VERTEX_SHADER
attribute vec3 a_v;
attribute vec2 a_vt;
attribute vec3 a_vn;
uniform mat4 u_projection_matrix;
uniform mat4 u_view_matrix;
uniform mat4 u_model_matrix;
uniform mat3 u_texture_matrix;
uniform mat4 u_light_matrix;

mat3 transpose(mat3 mat) {
    return mat3(
        vec3(mat[0].x, mat[1].x, mat[2].x),
        vec3(mat[0].y, mat[1].y, mat[2].y),
        vec3(mat[0].z, mat[1].z, mat[2].z));
}

void main() {
    v_normal = transpose(inverse(mat3(u_model_matrix))) * a_vn;
    v_uv = (u_texture_matrix * vec3(a_vt, 1.0)).xy;
    v_world_pos = (u_model_matrix * vec4(a_v, 1.0)).xyz;
    v_light_pos = u_light_matrix * vec4(v_world_pos, 1.0);
    v_eye_pos = (u_view_matrix * vec4(v_world_pos, 1.0)).xyz;
    gl_Position = u_projection_matrix * vec4(v_eye_pos, 1.0);
}
#endif

#ifdef FRAGMENT_SHADER
uniform vec4 u_color;
uniform float u_flashdark_angle;
uniform float u_flashdark_strength;
uniform vec3 u_flashdark_dir;
uniform vec3 u_flashdark_pos;
uniform sampler2D u_texture;
uniform sampler2D u_dark_texture;

uniform sampler2D u_shadow_map;
uniform ivec2 u_shadow_size;
uniform vec3 u_light_source;

void main() {
    float d = length(v_eye_pos);
    float fog_factor = 1.0 - exp(-d * 0.2) / exp(0.0);
    float flashdarked = smoothstep(cos(u_flashdark_angle), cos(u_flashdark_angle) + 0.1, dot(normalize(v_world_pos - u_flashdark_pos), u_flashdark_dir)) * u_flashdark_strength;

    // Shadow
    vec3 light_pos = v_light_pos.xyz / v_light_pos.w * 0.5 + 0.5;
    vec3 light_dir = normalize(u_light_source - v_world_pos);
    vec3 normal = normalize(v_normal);
    
    float cos = 1.0; // dot(light_dir, normal); // TODO: fix bias
    float bias = max(0.001, 0.01 * (1.0 - cos));

    vec2 texel_size = 1.0 / vec2(u_shadow_size);
    float shadow = 0.0;
    for (int i = -1; i <= 1; ++i) {
        for (int j = -1; j <= 1; ++j) {
            vec2 sample_pos = light_pos.xy + vec2(i, j) * texel_size;
            if (sample_pos.x <= 1.0 && sample_pos.x >= 0.0 && sample_pos.y <= 1.0 && sample_pos.y >= 0.0) {
                float pcf_depth = unpack4(texture2D(u_shadow_map, sample_pos));
                shadow += light_pos.z - bias > pcf_depth ? 1.0 : 0.0;
            } else {
                shadow += 1.0;
            }
        }
    }
    shadow /= 9.0;
    if (light_pos.z > 1.0) {
        shadow = 0.0;
    }
    flashdarked *= 1.0 - shadow;
    
    vec4 texture_color = (texture2D(u_dark_texture, v_uv) * flashdarked + texture2D(u_texture, v_uv) * (1.0 - flashdarked)) * vec4(u_color.xyz, 1.0);
    vec4 fog_color = vec4(0.0, 0.0, 0.0, texture_color.w);
    gl_FragColor = texture_color * (1.0 - fog_factor) + fog_color * fog_factor;

    if (gl_FragColor.w < 0.5) {
        discard;
    } else {
        gl_FragColor.w = u_color.w * (1.0 - shadow);
    }
}
#endif