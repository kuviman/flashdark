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
void main() {
    float d = length(v_eye_pos);
    gl_FragColor = pack4(d);
    // gl_FragColor = vec4(vec3(1.0 / d), 1.0); // Debug gray-scale visualizer
}
#endif