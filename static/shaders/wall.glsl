#ifdef VERTEX_SHADER
attribute vec3 a_pos;
uniform mat4 u_projection_matrix;
uniform mat4 u_view_matrix;
void main() {
    gl_Position = u_projection_matrix * u_view_matrix * vec4(a_pos, 1.0);
}
#endif

#ifdef FRAGMENT_SHADER
void main() {
    gl_FragColor = vec4(1.0, 1.0, 1.0, 1.0);
}
#endif