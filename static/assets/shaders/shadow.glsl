#ifdef VERTEX_SHADER
attribute vec3 a_v;
uniform mat4 u_projection_matrix;
uniform mat4 u_view_matrix;
uniform mat4 u_model_matrix;
void main() {
    gl_Position = u_projection_matrix * u_view_matrix * u_model_matrix * vec4(a_v, 1.0);
}
#endif

#ifdef FRAGMENT_SHADER
void main() {
    // if (gl_FragCoord.z > 1.0) {
    //     gl_FragColor = pack4(1.0);
    // } else {
    //     gl_FragColor = pack4(0.5);
    // }

    // gl_FragColor = vec4(vec3(gl_FragCoord.z / 10.0 - 0.9), 1.0);

    gl_FragColor = pack4(gl_FragCoord.z);
}
#endif