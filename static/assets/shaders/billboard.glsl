varying vec2 v_uv;
varying vec2 v_noise_uv;
varying vec3 v_eye_pos;

#ifdef VERTEX_SHADER
attribute vec2 a_pos;
uniform vec3 u_pos;
uniform vec2 u_size;
uniform mat4 u_projection_matrix;
uniform mat4 u_view_matrix;
void main() {
    v_uv = a_pos;
    v_noise_uv = a_pos * u_size;
    v_eye_pos = (u_view_matrix * vec4(u_pos + vec3(0.0, 0.0, a_pos.y * u_size.y), 1.0)).xyz + vec3((a_pos.x - 0.5) * u_size.x, 0.0, 0.0);
    gl_Position = u_projection_matrix * vec4(v_eye_pos, 1.0);
}
#endif

#ifdef FRAGMENT_SHADER
uniform sampler2D u_texture;
uniform sampler2D u_noise;
uniform float u_dissolve;
void main() {
    float d = length(v_eye_pos);
    float fog_factor = 1.0 - exp(-d * 0.2) / exp(0.0);
    vec4 texture_color = texture2D(u_texture, v_uv);
    vec4 fog_color = vec4(0.0, 0.0, 0.0, texture_color.w);
    gl_FragColor = texture_color * (1.0 - fog_factor) + fog_color * fog_factor;

    if (gl_FragColor.w < 0.5) {
        discard;
    }

    float n = texture2D(u_noise, v_noise_uv / 10.0).x;
    if (n < u_dissolve) {
        discard;
    }
}
#endif