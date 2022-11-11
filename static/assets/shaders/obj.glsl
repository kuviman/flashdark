varying vec2 v_uv;
varying vec3 v_eye_pos;
varying vec3 v_world_pos;

#ifdef VERTEX_SHADER
attribute vec3 a_v;
attribute vec2 a_vt;
attribute vec3 a_vn;
uniform mat4 u_projection_matrix;
uniform mat4 u_view_matrix;
uniform mat4 u_model_matrix;
uniform mat3 u_texture_matrix;
void main() {
    v_uv = (u_texture_matrix * vec3(a_vt, 1.0)).xy;
    v_world_pos = (u_model_matrix * vec4(a_v, 1.0)).xyz;
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
void main() {
    float d = length(v_eye_pos);
    float fog_factor = 1.0 - exp(-d * 0.2) / exp(0.0);
    float flashdarked = smoothstep(cos(u_flashdark_angle), cos(u_flashdark_angle) + 0.1, dot(normalize(v_world_pos - u_flashdark_pos), u_flashdark_dir)) * u_flashdark_strength;
    vec4 dark_color = texture2D(u_texture, v_uv) * (1.0 - u_flashdark_dark) + texture2D(u_dark_texture, v_uv) * u_flashdark_dark;
    vec4 light_color = texture2D(u_texture, v_uv) * u_ambient_light_color;
    vec4 texture_color = (dark_color * flashdarked + light_color * (1.0 - flashdarked)) * vec4(u_color.xyz, 1.0);
    vec4 fog_color = vec4(0.0, 0.0, 0.0, texture_color.w);
    gl_FragColor = texture_color * (1.0 - fog_factor) + fog_color * fog_factor;

    if (gl_FragColor.w < 0.5) {
        discard;
    } else {
        gl_FragColor.w = u_color.w;
    }
    gl_FragColor.xyz *= 1.0 - smoothstep(u_darkness, u_darkness + 3.0, v_world_pos.y);
}
#endif