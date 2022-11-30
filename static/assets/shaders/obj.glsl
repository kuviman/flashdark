varying float v_b;
varying vec2 v_uv;
varying vec3 v_eye_pos;
varying vec3 v_world_pos;
varying vec3 v_normal;

struct Light {
    vec3 pos;
    mat4 matrix;
    float n;
    float f;
    // sampler2D shadow_map; // https://stackoverflow.com/questions/25474625/glsl-sampler2d-in-struct
    ivec2 shadow_size;
    float intensity;
};
const int MAX_LIGHTS = 10; 
uniform Light u_lights[MAX_LIGHTS];
uniform sampler2D u_lights_shadow_maps[MAX_LIGHTS];

float normalize_depth(float depth) {
    float n = 0.1;
    float f = 50.0;
    float z_ndc = 2.0 * depth - 1.0;
    float z_eye = 2.0 * n * f / (f + n - z_ndc * (f - n));
    return z_eye;
}

float get_shadow_map_value(sampler2D shadow_map, vec2 pos) {
    return normalize_depth(unpack4(texture2D(shadow_map, pos)));
}

vec3 get_light_pos(Light light, vec3 pos) {
    vec4 v = light.matrix * vec4(pos, 1.0);
    v.xyz = v.xyz / v.w * 0.5 + 0.5;
    v.z = normalize_depth(v.z);
    return v.xyz;
}

const int SHADOWS_SOFT = 0;
uniform int u_lights_count;

varying vec4 v_light_pos[MAX_LIGHTS];

#ifdef VERTEX_SHADER
attribute vec3 a_v;
attribute float a_b;
attribute vec3 a_bv;
attribute vec2 a_vt;
attribute vec3 a_vn;
uniform float u_camera_rot;
uniform mat4 u_projection_matrix;
uniform mat4 u_view_matrix;
uniform mat4 u_model_matrix;
uniform mat3 u_texture_matrix;

// mat3 transpose(mat3 mat) {
//     return mat3(
//         vec3(mat[0].x, mat[1].x, mat[2].x),
//         vec3(mat[0].y, mat[1].y, mat[2].y),
//         vec3(mat[0].z, mat[1].z, mat[2].z));
// }

void main() {
    // // v_normal = transpose(inverse(mat3(u_model_matrix))) * a_vn;
    v_normal = mat3(u_model_matrix) * a_vn;
    v_uv = (u_texture_matrix * vec3(a_vt, 1.0)).xy;
    v_world_pos = (u_model_matrix * vec4(a_v + vec3(rotate(a_bv.xy, u_camera_rot), a_bv.z), 1.0)).xyz;
    // for (int i = 0; i < MAX_LIGHTS; ++i) {
    //     v_light_pos[i] = u_lights[i].matrix * vec4(v_world_pos, 1.0);
    // }
    v_eye_pos = (u_view_matrix * vec4(v_world_pos, 1.0)).xyz;
    v_normal *= dot(mat3(u_view_matrix) * v_normal, v_eye_pos) > 0.0 ? -1.0 : 1.0;
    gl_Position = u_projection_matrix * vec4(v_eye_pos, 1.0);
    v_b = a_b;
    // gl_Position = u_lights[0].matrix * vec4(v_world_pos, 1.0);
}
#endif

#ifdef FRAGMENT_SHADER
uniform vec4 u_color;
uniform float u_flashdark_angle;
uniform float u_flashdark_strength;
uniform vec3 u_flashdark_dir;
uniform vec3 u_flashdark_pos;
uniform float u_flashdark_dark;
uniform float u_darkness;
uniform vec4 u_ambient_light_color;
uniform sampler2D u_texture;
uniform sampler2D u_dark_texture;
uniform sampler2D u_noise;

float get_light_level(Light light, sampler2D light_shadow_map) {
    vec2 texel_size = 3.0 / vec2(light.shadow_size);
    
    vec3 light_pos = get_light_pos(light, v_world_pos); // v_light_pos[light].xyz / v_light_pos[light].w * 0.5 + 0.5;
    vec3 light_dir = normalize(light.pos - v_world_pos);
    vec3 normal = normalize(v_normal);
    
    float cos = dot(light_dir, normal); // TODO: fix bias
    if (cos < 0.0 && v_b < 0.5) {
        return 0.0;
    }
    float bias = max(0.01, 0.1 * (1.0 - cos));

    float l_shadow = 0.0;
    for (int i = -SHADOWS_SOFT; i <= SHADOWS_SOFT; ++i) {
        for (int j = -SHADOWS_SOFT; j <= SHADOWS_SOFT; ++j) {
            // vec2 n = vec2(cnoise(gl_FragCoord.xy), cnoise(gl_FragCoord.xy + vec2(123.0, 456.0)));
            vec2 sample_pos = light_pos.xy + vec2(i, j) * texel_size;
            // sample_pos += texture2D(u_noise, gl_FragCoord.xy / 2000.0).xy * texel_size * 5.0;
            if (sample_pos.x <= 1.0 && sample_pos.x >= 0.0 && sample_pos.y <= 1.0 && sample_pos.y >= 0.0) {
                float pcf_depth = get_shadow_map_value(light_shadow_map, sample_pos);
                l_shadow += light_pos.z - bias > pcf_depth ? 1.0 : 0.0;
            } else {
                l_shadow += 1.0;
            }
        }
    }
    l_shadow /= (2.0 * float(SHADOWS_SOFT) + 1.0) * (2.0 * float(SHADOWS_SOFT) + 1.0);
    if (light_pos.z < 0.0) {
        l_shadow = 1.0;
    }
    return (1.0 - l_shadow) * light.intensity;// * cos;
}

void main() {
    // vec4 a = u_lights[0].matrix * vec4(v_world_pos, 1.0);
    // gl_FragColor = vec4(a.xyz / a.w * 0.5 + 0.5, 1.0);
    // gl_FragColor = vec4(get_light_pos(u_lights[0], v_world_pos), 1.0);
    // gl_FragColor = vec4(0.0, 1.0, 0.0, 1.0);
    // return;

    float d = length(v_eye_pos);
    float fog_factor = 1.0 - exp(-d * 0.2) / exp(0.0);
    float flashdarked = smoothstep(cos(u_flashdark_angle), cos(u_flashdark_angle) + 0.1, dot(normalize(v_world_pos - u_flashdark_pos), u_flashdark_dir)) * u_flashdark_strength;

    // Shadow
    float light_level = 0.0;
    for (int light = 1; light < MAX_LIGHTS; ++light) {
        if (light >= u_lights_count) { break; }
        light_level += get_light_level(u_lights[light], u_lights_shadow_maps[light]);
    }
    float flashlight_level = get_light_level(u_lights[0], u_lights_shadow_maps[0]);
    light_level += (1.0 - u_flashdark_dark) * flashlight_level;
    // Ambient
    // light_level = max(0.05, light_level);
    vec4 light_color = u_ambient_light_color * (1.0 - light_level) + vec4(1.0, 1.0, 1.0, 1.0) * light_level;
    
    flashdarked *= min(1.0, flashlight_level);
    
    vec4 normal_color = texture2D(u_texture, v_uv) * light_color;
    vec4 dark_color = texture2D(u_texture, v_uv) * (1.0 - u_flashdark_dark) + texture2D(u_dark_texture, v_uv) * u_flashdark_dark;
    dark_color.xyz *= light_color.xyz * 0.75;
    vec4 texture_color = (dark_color * flashdarked + normal_color * (1.0 - flashdarked)) * vec4(u_color.xyz, 1.0);
    vec4 fog_color = vec4(0.0, 0.0, 0.0, texture_color.w);
    gl_FragColor = texture_color * (1.0 - fog_factor) + fog_color * fog_factor;

    if (gl_FragColor.w < 0.5) {
        discard;
    } else {
        gl_FragColor.w = u_color.w;
    }
    gl_FragColor.xyz *= 1.0 - smoothstep(u_darkness, u_darkness + 3.0, v_world_pos.y);
    // gl_FragColor.xyz *= light_color.xyz;
}
#endif