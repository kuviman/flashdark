varying vec3 v_world_pos;
varying vec2 v_uv;
varying vec3 v_eye_pos;

#ifdef VERTEX_SHADER
attribute vec3 a_pos;
attribute vec2 a_uv;
uniform mat4 u_projection_matrix;
uniform mat4 u_view_matrix;
void main() {
    v_uv = a_uv;
    v_world_pos = a_pos;
    v_eye_pos = (u_view_matrix * vec4(v_world_pos, 1.0)).xyz;
    gl_Position = u_projection_matrix * vec4(v_eye_pos, 1.0);
}
#endif

#ifdef FRAGMENT_SHADER
uniform sampler2D u_texture;

float light_at(vec3 pos) {
    vec3 light_pos = vec3(1.0, 1.0, 1.0);
    vec3 light_dir = normalize(vec3(-1.0, -1.0, -1.0));
    float light_angle = 0.5;
    float a = smoothstep(cos(light_angle), cos(light_angle) + 0.1, dot(light_dir, normalize(pos - light_pos)));
    return a * 0.9 + 0.1;
    // if (dot(light_dir, normalize(pos - light_pos)) < cos(light_angle)) {
    //     return 0.0;
    // }
    // float d = length(pos - light_pos);
    // return 1.0; //exp(-d * 2.0);
}

void main() {
    float d = length(v_eye_pos);
    float fog_factor = 1.0 - exp(-d);
    vec4 fog_color = vec4(0.0, 0.0, 0.0, 1.0);
    vec4 texture_color = texture2D(u_texture, v_uv);

    float light = light_at(v_world_pos);
    
    vec4 color = texture_color * light + fog_color * (1.0 - light);

    gl_FragColor = color * (1.0 - fog_factor) + fog_color * fog_factor;
}
#endif