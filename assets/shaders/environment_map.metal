#include <metal_stdlib>
#import "../../shader_types/shader_types.h"

using namespace metal;

struct SkyboxVertexIn {
  float4 position [[attribute(Position)]];
  // float4 normal [[attribute(Normal)]];
};

struct SkyboxVertexOut {
  float4 position [[position]];
  float3 texCoords;
};

vertex SkyboxVertexOut vertex_skybox(const SkyboxVertexIn vertexIn [[stage_in]], constant float4x4 &vp [[buffer(1)]]) {
  SkyboxVertexOut out {
      .position = (vp * vertexIn.position).xyww,
      // .position = uniforms.projectionMatrix * uniforms.viewMatrix * uniforms.modelMatrix * vertexIn.position,
      .texCoords = vertexIn.position.xyz
  };
  return out;
}

fragment half4 fragment_skybox(SkyboxVertexOut in [[stage_in]], constant Uniforms &uniforms [[buffer(1)]], 
                                    texturecube<half> cubeMap [[texture(CubeMap)]]) {
  float3 texCoords = float3(in.texCoords.x, in.texCoords.y, -in.texCoords.z);
  constexpr sampler defaultSampler (filter::linear);
  return cubeMap.sample(defaultSampler, texCoords);
  /* return half4(1, 1, 0, 1); */
}
