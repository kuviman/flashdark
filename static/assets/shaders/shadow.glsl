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
uniform ivec2 u_shadow_size;

void main() {
    float d = length(gl_FragCoord.xy / vec2(u_shadow_size) - 0.5);
    float depth = d < 0.5 ? gl_FragCoord.z : 0.0;
    gl_FragColor = pack4(depth);
}
#endif