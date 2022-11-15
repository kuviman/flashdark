//	Classic Perlin 2D Noise 
//	by Stefan Gustavson
//
vec4 permute(vec4 x){return mod(((x*34.0)+1.0)*x, 289.0);}
vec2 fade(vec2 t) {return t*t*t*(t*(t*6.0-15.0)+10.0);}

float cnoise(vec2 P){
  vec4 Pi = floor(P.xyxy) + vec4(0.0, 0.0, 1.0, 1.0);
  vec4 Pf = fract(P.xyxy) - vec4(0.0, 0.0, 1.0, 1.0);
  Pi = mod(Pi, 289.0); // To avoid truncation effects in permutation
  vec4 ix = Pi.xzxz;
  vec4 iy = Pi.yyww;
  vec4 fx = Pf.xzxz;
  vec4 fy = Pf.yyww;
  vec4 i = permute(permute(ix) + iy);
  vec4 gx = 2.0 * fract(i * 0.0243902439) - 1.0; // 1/41 = 0.024...
  vec4 gy = abs(gx) - 0.5;
  vec4 tx = floor(gx + 0.5);
  gx = gx - tx;
  vec2 g00 = vec2(gx.x,gy.x);
  vec2 g10 = vec2(gx.y,gy.y);
  vec2 g01 = vec2(gx.z,gy.z);
  vec2 g11 = vec2(gx.w,gy.w);
  vec4 norm = 1.79284291400159 - 0.85373472095314 * 
    vec4(dot(g00, g00), dot(g01, g01), dot(g10, g10), dot(g11, g11));
  g00 *= norm.x;
  g01 *= norm.y;
  g10 *= norm.z;
  g11 *= norm.w;
  float n00 = dot(g00, vec2(fx.x, fy.x));
  float n10 = dot(g10, vec2(fx.y, fy.y));
  float n01 = dot(g01, vec2(fx.z, fy.z));
  float n11 = dot(g11, vec2(fx.w, fy.w));
  vec2 fade_xy = fade(Pf.xy);
  vec2 n_x = mix(vec2(n00, n01), vec2(n10, n11), fade_xy.x);
  float n_xy = mix(n_x.x, n_x.y, fade_xy.y);
  return 2.3 * n_xy;
}

// ^ Copypasted from https://gist.github.com/patriciogonzalezvivo/670c22f3966e662d2f83

varying vec2 v_uv;
varying vec3 v_eye_pos;
varying vec3 v_world_pos;
varying vec3 v_normal;

struct Light {
    vec3 pos;
    mat4 matrix;
    float n;
    float f;
    sampler2D shadow_map;
    ivec2 shadow_size;
    float intensity;
};

float fix_z(Light light, float x) {
    return x;
    // float n = 0.1;
    // float f = 50.0;
    // float z_ndc = 2.0 * x - 1.0;
    // return 2.0 * n * f / (f + n - z_ndc * (f - n));
}

float get_shadow_map_value(Light light, vec2 pos) {
    float v = unpack4(texture2D(light.shadow_map, pos));
    return fix_z(light, v);
}

vec3 get_light_pos(Light light, vec3 pos) {
    vec4 v = light.matrix * vec4(pos, 1.0);
    vec3 p = v.xyz / v.w * 0.5 + 0.5;
    p.z = fix_z(light, p.z);
    return p;
}

const int SHADOWS_SOFT = 0;
const int MAX_LIGHTS = 25;
uniform Light u_lights[MAX_LIGHTS];
uniform int u_lights_count;

varying vec4 v_light_pos[MAX_LIGHTS];

#ifdef VERTEX_SHADER
attribute vec3 a_v;
attribute vec2 a_vt;
attribute vec3 a_vn;
uniform mat4 u_projection_matrix;
uniform mat4 u_view_matrix;
uniform mat4 u_model_matrix;
uniform mat3 u_texture_matrix;

mat3 transpose(mat3 mat) {
    return mat3(
        vec3(mat[0].x, mat[1].x, mat[2].x),
        vec3(mat[0].y, mat[1].y, mat[2].y),
        vec3(mat[0].z, mat[1].z, mat[2].z));
}

void main() {
    // v_normal = transpose(inverse(mat3(u_model_matrix))) * a_vn;
    v_normal = mat3(u_model_matrix) * a_vn;
    v_uv = (u_texture_matrix * vec3(a_vt, 1.0)).xy;
    v_world_pos = (u_model_matrix * vec4(a_v, 1.0)).xyz;
    // for (int i = 0; i < MAX_LIGHTS; ++i) {
    //     v_light_pos[i] = u_lights[i].matrix * vec4(v_world_pos, 1.0);
    // }
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
uniform float u_flashdark_dark;
uniform float u_darkness;
uniform vec4 u_ambient_light_color;
uniform sampler2D u_texture;
uniform sampler2D u_dark_texture;
uniform sampler2D u_noise;

void main() {
    float d = length(v_eye_pos);
    float fog_factor = 1.0 - exp(-d * 0.2) / exp(0.0);
    float flashdarked = smoothstep(cos(u_flashdark_angle), cos(u_flashdark_angle) + 0.1, dot(normalize(v_world_pos - u_flashdark_pos), u_flashdark_dir)) * u_flashdark_strength;

    // Shadow
    float light_level = 0.0;
    for (int light = 0; light < MAX_LIGHTS; ++light) {
        if (light >= u_lights_count) { break; }
        vec2 texel_size = 3.0 / vec2(u_lights[light].shadow_size);
        
        vec3 light_pos = get_light_pos(u_lights[light], v_world_pos); // v_light_pos[light].xyz / v_light_pos[light].w * 0.5 + 0.5;
        vec3 light_dir = normalize(u_lights[0].pos - v_world_pos);
        vec3 normal = normalize(v_normal);
        
        float cos = max(dot(light_dir, normal), 0.0); // TODO: fix bias
        float bias = 0.01; //max(0.005, 0.01 * (1.0 - cos));

        float l_shadow = 0.0;
        for (int i = -SHADOWS_SOFT; i <= SHADOWS_SOFT; ++i) {
            for (int j = -SHADOWS_SOFT; j <= SHADOWS_SOFT; ++j) {
                // vec2 n = vec2(cnoise(gl_FragCoord.xy), cnoise(gl_FragCoord.xy + vec2(123.0, 456.0)));
                vec2 n = texture2D(u_noise, gl_FragCoord.xy / 2000.0).xy;
                vec2 sample_pos = light_pos.xy + vec2(i, j) * texel_size;
                sample_pos += n * texel_size * 5.0;
                if (sample_pos.x <= 1.0 && sample_pos.x >= 0.0 && sample_pos.y <= 1.0 && sample_pos.y >= 0.0) {
                    float pcf_depth = get_shadow_map_value(u_lights[light], sample_pos);
                    l_shadow += light_pos.z - bias > pcf_depth ? 1.0 : 0.0;
                } else {
                    l_shadow += 1.0;
                }
            }
        }
        l_shadow /= (2.0 * float(SHADOWS_SOFT) + 1.0) * (2.0 * float(SHADOWS_SOFT) + 1.0);
        // if (light_pos.z > 1.0) {
        //     l_shadow = 0.0;
        // }
        light_level += (1.0 - l_shadow) * u_lights[light].intensity;// * cos;
    }
    // Ambient
    light_level = max(0.05, light_level);
    vec4 light_color = u_ambient_light_color * (1.0 - light_level) + vec4(1.0, 1.0, 1.0, 1.0) * light_level;
    
    flashdarked *= min(1.0, light_level);
    
    vec4 normal_color = texture2D(u_texture, v_uv);
    vec4 dark_color = texture2D(u_texture, v_uv) * (1.0 - u_flashdark_dark) + texture2D(u_dark_texture, v_uv) * u_flashdark_dark;
    vec4 texture_color = (dark_color * flashdarked + normal_color * (1.0 - flashdarked)) * vec4(u_color.xyz, 1.0);
    vec4 fog_color = vec4(0.0, 0.0, 0.0, texture_color.w);
    gl_FragColor = texture_color * (1.0 - fog_factor) + fog_color * fog_factor;

    if (gl_FragColor.w < 0.5) {
        discard;
    } else {
        gl_FragColor.w = u_color.w;
    }
    gl_FragColor.xyz *= 1.0 - smoothstep(u_darkness, u_darkness + 3.0, v_world_pos.y);
    gl_FragColor.xyz *= light_color.xyz;
}
#endif