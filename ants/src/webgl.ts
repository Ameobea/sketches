/**
 * Much of this code was taken or adapted from https://webglfundamentals.org/ and is used under
 * the terms of the license attached below:
 *
 * Copyright 2012, Gregg Tavares.
 * All rights reserved.
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions are
 * met:
 *
 *     * Redistributions of source code must retain the above copyright
 * notice, this list of conditions and the following disclaimer.
 *     * Redistributions in binary form must reproduce the above
 * copyright notice, this list of conditions and the following disclaimer
 * in the documentation and/or other materials provided with the
 * distribution.
 *     * Neither the name of Gregg Tavares. nor the names of his
 * contributors may be used to endorse or promote products derived from
 * this software without specific prior written permission.
 *
 * THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
 * "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
 * LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
 * A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
 * OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
 * SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
 * LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
 * DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
 * THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
 * (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
 * OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
 */

import vertexShaderSrc from './shaders/vert.glsl';
import fragShaderSrc from './shaders/frag.glsl';

import { getCanvas } from './canvas';

const canvasWebGL: HTMLCanvasElement = getCanvas() as HTMLCanvasElement;

const gl = (() => {
  const ctx = canvasWebGL.getContext('webgl');
  if (!ctx) {
    const errMsg = 'Unable to create WebGL rendering context; this application cannot run!';
    alert(errMsg);
    throw Error(errMsg);
  }
  return ctx;
})();

/**
 * Compiles either a shader of type `gl.VERTEX_SHADER` or `gl.FRAGMENT_SHADER`.
 */
const createShader = (sourceCode: string, type: number): WebGLShader => {
  const shader = gl.createShader(type)!;
  gl.shaderSource(shader, sourceCode);
  gl.compileShader(shader);

  if (!shader || !gl.getShaderParameter(shader, gl.COMPILE_STATUS)) {
    const info = shader && gl.getShaderInfoLog(shader);
    throw 'Could not compile WebGL program. \n\n' + info;
  }
  return shader;
};

const localState: { [key: string]: any } = {};

export const initWebGL = (): { canvasHeight: number; canvasWidth: number } => {
  console.log('Initializing WebGL');
  localState.backgroundProgram = gl.createProgram();
  const backgroundProgram = localState.backgroundProgram;

  gl.attachShader(backgroundProgram, createShader(vertexShaderSrc, gl.VERTEX_SHADER));
  gl.attachShader(backgroundProgram, createShader(fragShaderSrc, gl.FRAGMENT_SHADER));

  gl.linkProgram(backgroundProgram);

  // look up where the vertex data needs to go.
  localState.positionLocation = gl.getAttribLocation(backgroundProgram, 'a_position');
  localState.texcoordLocation = gl.getAttribLocation(backgroundProgram, 'a_texcoord');

  // lookup uniforms
  localState.matrixLocation = gl.getUniformLocation(backgroundProgram, 'u_matrix');
  localState.textureMatrixLocation = gl.getUniformLocation(backgroundProgram, 'u_textureMatrix');
  localState.textureLocation = gl.getUniformLocation(backgroundProgram, 'u_texture');

  // scale factor
  localState.scaleFactorLocation = gl.getUniformLocation(backgroundProgram, 'f_scalefactor');

  // Create a buffer.
  localState.positionBuffer = gl.createBuffer();
  gl.bindBuffer(gl.ARRAY_BUFFER, localState.positionBuffer);

  // Put a unit quad in the buffer
  localState.positions = [0, 0, 0, 1, 1, 0, 1, 0, 0, 1, 1, 1];
  gl.bufferData(gl.ARRAY_BUFFER, new Float32Array(localState.positions), gl.STATIC_DRAW);

  // Create a buffer for texture coords
  localState.texcoordBuffer = gl.createBuffer();
  gl.bindBuffer(gl.ARRAY_BUFFER, localState.texcoordBuffer);

  // Put texcoords in the buffer
  localState.texcoords = [0, 0, 0, 1, 1, 0, 1, 0, 0, 1, 1, 1];
  gl.bufferData(gl.ARRAY_BUFFER, new Float32Array(localState.texcoords), gl.STATIC_DRAW);

  if (!backgroundProgram) {
    throw 'Unable to create WebGL program';
  }

  if (!gl.getProgramParameter(backgroundProgram, gl.LINK_STATUS)) {
    throw `Could not compile WebGL program. \n\n'${gl.getProgramInfoLog(backgroundProgram)}`;
  }

  gl.viewport(0, 0, gl.canvas.width, gl.canvas.height);

  console.log('WebGL Initialized');
  return { canvasHeight: gl.canvas.height, canvasWidth: gl.canvas.width };
};

export const createBackgroundTexture = async (textureData: Uint8Array) => {
  const textureSize = Math.sqrt(textureData.length) / 2;
  localState.textureSize = textureSize;
  // Create an initialize a WebGL texture
  const texture = gl.createTexture();
  if (!texture) {
    throw 'Unable to create WebGL texture';
  }
  localState.backgroundTexture = texture;
  localState.backgroundHeight = textureSize;
  localState.backgroundWidth = textureSize;
  gl.bindTexture(gl.TEXTURE_2D, texture);

  // Populate the texture with the pixel data generated in Wasm
  gl.texImage2D(
    gl.TEXTURE_2D,
    0,
    gl.RGBA,
    textureSize,
    textureSize,
    0,
    gl.RGBA,
    gl.UNSIGNED_BYTE,
    textureData
  );

  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
  // Linear interpolation when shrinking the texture
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
  // Pick the nearest pixel from the underlying texture when magnifying (which we're probably
  // always going to be doing)
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.NEAREST);
};

export const render = (scaleFactor: number, offsetX: number, offsetY: number) => {
  gl.bindTexture(gl.TEXTURE_2D, localState.backgroundTexture);

  // Tell WebGL to use our shader program pair
  gl.useProgram(localState.backgroundProgram);

  gl.bindBuffer(gl.ARRAY_BUFFER, localState.positionBuffer);
  gl.enableVertexAttribArray(localState.positionLocation);
  gl.vertexAttribPointer(localState.positionLocation, 2, gl.FLOAT, false, 0, 0);
  gl.bindBuffer(gl.ARRAY_BUFFER, localState.texcoordBuffer);
  gl.enableVertexAttribArray(localState.texcoordLocation);
  gl.vertexAttribPointer(localState.texcoordLocation, 2, gl.FLOAT, false, 0, 0);

  // Set the matrix.
  gl.uniformMatrix4fv(
    localState.matrixLocation,
    false,
    new Float32Array([2, 0, 0, 0, 0, -2, 0, 0, 0, 0, -1, 0, -1, 1, -0, 1])
  );

  // Set the scale factor
  gl.uniform1f(localState.scaleFactorLocation, scaleFactor);

  const texMatrix = new Float32Array([
    gl.canvas.width / localState.textureSize,
    0,
    0,
    0,
    0,
    gl.canvas.height / localState.textureSize,
    0,
    0,
    0,
    0,
    1,
    0,
    0,
    0,
    0,
    1,
  ]);

  // Set the texture matrix.
  gl.uniformMatrix4fv(localState.textureMatrixLocation, false, texMatrix);

  // Tell the shader to get the texture from texture unit 0
  gl.uniform1i(localState.textureLocation, 0);

  // draw the quad (2 triangles, 6 vertices)
  gl.drawArrays(gl.TRIANGLES, 0, 6);
};
