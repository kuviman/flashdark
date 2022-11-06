varying vec2 v_uv;
varying vec3 v_eye_pos;

#ifdef VERTEX_SHADER
attribute vec2 a_pos;
uniform vec3 u_pos;
uniform vec2 u_size;
uniform float u_rot;
uniform mat4 u_projection_matrix;
uniform mat4 u_view_matrix;
void main() {
    v_uv = a_pos;
    vec2 local_pos = rotate(vec2(a_pos.x - 0.5, a_pos.y - 0.5) * u_size, u_rot);
    vec3 world_pos = u_pos + vec3(local_pos, 0.0);
    v_eye_pos = (u_view_matrix * vec4(world_pos, 1.0)).xyz;
    gl_Position = u_projection_matrix * vec4(v_eye_pos, 1.0);
}
#endif

#ifdef FRAGMENT_SHADER
uniform sampler2D u_texture;
void main() {
    float d = length(v_eye_pos);
    float fog_factor = 1.0 - exp(-d) / exp(0.0);
    vec4 texture_color = texture2D(u_texture, v_uv);
    vec4 fog_color = vec4(0.0, 0.0, 0.0, texture_color.w);
    gl_FragColor = texture_color * (1.0 - fog_factor) + fog_color * fog_factor;

    if (gl_FragColor.w < 0.5) {
        discard;
    }
}
#endif