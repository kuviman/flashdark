varying vec2 v_uv;
varying vec3 v_eye_pos;

#ifdef VERTEX_SHADER
attribute vec3 a_pos;
attribute vec2 a_uv;
uniform mat4 u_projection_matrix;
uniform mat4 u_view_matrix;
void main() {
    v_uv = a_uv;
    v_eye_pos = (u_view_matrix * vec4(a_pos, 1.0)).xyz;
    gl_Position = u_projection_matrix * vec4(v_eye_pos, 1.0);
}
#endif

#ifdef FRAGMENT_SHADER
uniform sampler2D u_texture;
void main() {
    float d = length(v_eye_pos);
    float fog_factor = 1.0 - exp(-d) / exp(0.0);
    vec4 fog_color = vec4(0.0, 0.0, 0.0, 1.0);
    vec4 texture_color = texture2D(u_texture, v_uv);
    gl_FragColor = texture_color * (1.0 - fog_factor) + fog_color * fog_factor;
}
#endif