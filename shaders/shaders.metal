#include <metal_stdlib>
#import "../shader_types/shader_types.h"

using namespace metal;

struct VertexIn {
  float4 position [[attribute(0)]];
  float3 normal [[attribute(1)]];
};

struct VertexOut {
  float4 position [[position]];
  float3 worldPosition;
  float3 worldNormal;
};

vertex VertexOut vertex_main(VertexIn vertexIn [[stage_in]], constant Uniforms &uniforms [[buffer(2)]]) {
  VertexOut out {
    .position = uniforms.projectionMatrix * uniforms.viewMatrix * uniforms.modelMatrix * vertexIn.position,
    .worldPosition = (uniforms.modelMatrix * vertexIn.position).xyz,
    .worldNormal = uniforms.normalMatrix * vertexIn.normal
  };
  return out;
}

fragment float4 fragment_main(VertexOut in [[stage_in]],
  constant Light *lights [[buffer(3)]],
  constant FragementUniforms &fragmentUniforms [[buffer(4)]]) {

  float3 baseColor = float3(1, 1, 1);
  float3 diffuseColor = 0;

  float3 normalDirection = normalize(in.worldNormal);
  for (unsigned int i = 0; i < fragmentUniforms.lightCount; i++) {
    Light light = lights[i];
    if (light.type == Sunlight) {
      float3 lightDirection = normalize(-light.position);
      float diffuseIntensity = saturate(-dot(lightDirection, normalDirection));
      // float diffuseIntensity = -dot(lightDirection, normalDirection) * 0.5 + 0.5;
      diffuseColor += light.color * baseColor * diffuseIntensity;
    }
  }

  float3 color = diffuseColor;
  return float4(color, 1);
}