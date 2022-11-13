varying vec2 v_uv;

#ifdef VERTEX_SHADER
attribute vec3 a_v;
attribute vec2 a_vt;
uniform mat4 u_projection_matrix;
uniform mat4 u_view_matrix;
uniform mat4 u_model_matrix;
uniform mat3 u_texture_matrix;
void main() {
    v_uv = (u_texture_matrix * vec3(a_vt, 1.0)).xy;
    gl_Position = u_projection_matrix * u_view_matrix * u_model_matrix * vec4(a_v, 1.0);
}
#endif

#ifdef FRAGMENT_SHADER
uniform ivec2 u_shadow_size;
uniform sampler2D u_texture;

void main() {
    vec4 tex_color = texture2D(u_texture, v_uv);
    if (tex_color.a < 0.5) {
        discard;
    } 
    float d = length(gl_FragCoord.xy / vec2(u_shadow_size) - 0.5);
    float depth = d < 0.5 ? gl_FragCoord.z : 0.0;
    gl_FragColor = pack4(depth);
}
#endif