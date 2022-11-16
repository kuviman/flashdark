varying vec2 v_uv;

#ifdef VERTEX_SHADER
attribute vec3 a_v;
attribute vec2 a_vt;
uniform mat4 u_projection_matrix;
uniform mat4 u_view_matrix;
uniform mat4 u_model_matrix;

void main() {
    v_uv = a_vt;
    gl_Position = u_projection_matrix * u_view_matrix * u_model_matrix * vec4(a_v, 1.0);
}
#endif

#ifdef FRAGMENT_SHADER
uniform sampler2D u_texture;

void main() {
    gl_FragColor = texture2D(u_texture, v_uv);
}
#endif