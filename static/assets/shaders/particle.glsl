varying vec2 v_uv;
varying vec4 v_color;

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

float get_shadow_map_value(sampler2D shadow_map, vec2 pos) {
    return unpack4(texture2D(shadow_map, pos));
}

vec3 get_light_pos(Light light, vec3 pos) {
    vec4 v = light.matrix * vec4(pos, 1.0);
    return v.xyz / v.w * 0.5 + 0.5;
}

const int SHADOWS_SOFT = 0;
uniform int u_lights_count;

varying vec4 v_light_pos[MAX_LIGHTS];

#ifdef VERTEX_SHADER
attribute vec3 a_v;
attribute vec2 a_vt;
attribute vec3 i_pos;
attribute float i_life;
attribute float i_size;
uniform mat4 u_projection_matrix;
uniform mat4 u_view_matrix;
uniform vec4 u_color;
uniform float u_flashdark_angle;
uniform float u_flashdark_strength;
uniform vec3 u_flashdark_dir;
uniform vec3 u_flashdark_pos;
uniform float u_flashdark_dark;
uniform float u_darkness;
uniform vec4 u_ambient_light_color;
uniform sampler2D u_noise;

void main() {
    v_uv = a_vt;
    vec3 v_world_pos = i_pos;
    vec3 v_eye_pos = (u_view_matrix * vec4(v_world_pos, 1.0)).xyz;
    gl_Position = u_projection_matrix * vec4(v_eye_pos + a_v * i_size, 1.0);

    float d = length(v_eye_pos);
    float fog_factor = 1.0 - exp(-d * 0.2) / exp(0.0);
    float flashdarked = smoothstep(cos(u_flashdark_angle), cos(u_flashdark_angle) + 0.1, dot(normalize(v_world_pos - u_flashdark_pos), u_flashdark_dir)) * u_flashdark_strength;

    // Shadow
    float light_level = 0.0;
    for (int light = 0; light < MAX_LIGHTS; ++light) {
        if (light >= u_lights_count) { break; }
    //     vec2 texel_size = 3.0 / vec2(u_lights[light].shadow_size);
        
        vec3 light_pos = get_light_pos(u_lights[light], v_world_pos); // v_light_pos[light].xyz / v_light_pos[light].w * 0.5 + 0.5;
        vec3 light_dir = normalize(u_lights[light].pos - v_world_pos);
        // vec3 normal = normalize(v_normal);
        
        // float cos = max(dot(light_dir, normal), 0.0); // TODO: fix bias
        float bias = 0.01; //max(0.005, 0.01 * (1.0 - cos));

        float l_shadow = 0.0;
        vec2 sample_pos = light_pos.xy;// + vec2(i, j) * texel_size;
        if (sample_pos.x <= 1.0 && sample_pos.x >= 0.0 && sample_pos.y <= 1.0 && sample_pos.y >= 0.0) {
            float pcf_depth = get_shadow_map_value(u_lights_shadow_maps[light], sample_pos);
            l_shadow += light_pos.z - bias > pcf_depth ? 1.0 : 0.0;
        } else {
            l_shadow += 1.0;
        }
        if (light_pos.z > 1.0) {
            l_shadow = 1.0;
        }
        light_level += (1.0 - l_shadow) * u_lights[light].intensity;// * cos;
    }
    // Ambient
    light_level = max(0.05, light_level);
    vec4 light_color = u_ambient_light_color * (1.0 - light_level) + vec4(1.0, 1.0, 1.0, 1.0) * light_level;
    
    flashdarked *= min(1.0, light_level);
    
    vec4 normal_color = u_color;
    vec4 dark_color = u_color;
    vec4 texture_color = (dark_color * flashdarked + normal_color * (1.0 - flashdarked)) * vec4(u_color.xyz, 1.0);
    vec4 fog_color = vec4(0.0, 0.0, 0.0, texture_color.w);
    v_color = texture_color * (1.0 - fog_factor) + fog_color * fog_factor;
    v_color.xyz *= light_color.xyz;
    v_color.w *= 1.0 - (i_life * 2.0 - 1.0) * (i_life * 2.0 - 1.0);
    // v_color *= vec4(0.5,0.5,0.5,0.6);
}
#endif

#ifdef FRAGMENT_SHADER

uniform sampler2D u_texture;

void main() {
    gl_FragColor = texture2D(u_texture, v_uv) * v_color;
}
#endif