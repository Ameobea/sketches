precision mediump float;

varying vec2 v_texcoord;

uniform float f_scalefactor;
uniform sampler2D u_texture;

void main() {
    gl_FragColor = texture2D(u_texture, v_texcoord / f_scalefactor);
}
