varying vec2 v_uv;

#ifdef VERTEX_SHADER
attribute vec2 a_pos;
uniform vec3 u_pos;
uniform vec2 u_size;
uniform mat4 u_projection_matrix;
uniform mat4 u_view_matrix;
void main() {
    v_uv = a_pos;
    vec3 pos = (u_view_matrix * vec4(u_pos, 1.0)).xyz + vec3((a_pos - 0.5) * u_size, 0.0);
    gl_Position = u_projection_matrix * vec4(pos, 1.0);
}
#endif

#ifdef FRAGMENT_SHADER
uniform sampler2D u_texture;
uniform vec4 u_color;
void main() {
    gl_FragColor = texture2D(u_texture, v_uv) * u_color;
    gl_FragColor.w *= 0.3;
}
#endif