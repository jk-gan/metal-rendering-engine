#include <metal_stdlib>
#import "../shader_types/shader_types.h"

using namespace metal;

struct VertexIn {
  float4 position [[attribute(Position)]];
  float3 normal [[attribute(Normal)]];
  float2 uv [[attribute(UV)]];
};

struct VertexOut {
  float4 position [[position]];
  float3 worldPosition;
  float3 worldNormal;
  float2 uv;
};

vertex VertexOut vertex_main(VertexIn vertexIn [[stage_in]], constant Uniforms &uniforms [[buffer(BufferIndexUniforms)]]) {
  VertexOut out {
    .position = uniforms.projectionMatrix * uniforms.viewMatrix * uniforms.modelMatrix * vertexIn.position,
    .worldPosition = (uniforms.modelMatrix * vertexIn.position).xyz,
    .worldNormal = uniforms.normalMatrix * vertexIn.normal,
    .uv = vertexIn.uv
  };
  return out;
}

fragment float4 fragment_main(VertexOut in [[stage_in]],
  constant Light *lights [[buffer(BufferIndexLights)]],
  constant FragementUniforms &fragmentUniforms [[buffer(BufferIndexFragmentUniforms)]]) {

  float3 baseColor = float3(1, 1, 1);
  float3 diffuseColor = 0;
  float3 ambientColor = 0;
  float3 specularColor = 0;
  float materialShininess = 32;
  float3 materialSpecularColor = float3(1, 1, 1);

  float3 normalDirection = normalize(in.worldNormal);
  for (unsigned int i = 0; i < fragmentUniforms.lightCount; i++) {
    Light light = lights[i];
    if (light.type == Sunlight) {
      float3 lightDirection = normalize(-light.position);
      float diffuseIntensity = saturate(-dot(lightDirection, normalDirection));
      // float diffuseIntensity = -dot(lightDirection, normalDirection) * 0.5 + 0.5;
      diffuseColor += light.color * baseColor * diffuseIntensity;

      if (diffuseIntensity > 0) {
        float3 reflection = reflect(lightDirection, normalDirection);
        float3 cameraDirection = normalize(in.worldPosition - fragmentUniforms.cameraPosition);
        float specularIntensity = pow(saturate(-dot(reflection, cameraDirection)), materialShininess);
        specularColor += light.specularColor * materialSpecularColor * specularIntensity;
      }
    } else if (light.type == Ambientlight) {
      ambientColor += light.color * light.intensity;
    } else if (light.type == Pointlight) {
      float d = distance(light.position, in.worldPosition);
      float3 lightDirection = normalize(in.worldPosition - light.position);
      float attenuation = 1.0 / (light.attenuation.x + light.attenuation.y * d + light.attenuation.z * d * d);
      float diffuseIntensity = saturate(-dot(lightDirection, normalDirection));
      float3 color = light.color * baseColor * diffuseIntensity;
      color *= attenuation;
      diffuseColor += color;
    } else if (light.type == Spotlight) {
      float d = distance(light.position, in.worldPosition);
      float3 lightDirection = normalize(in.worldPosition - light.position);
      float3 coneDirection = normalize(light.coneDirection);
      float spotResult = dot(lightDirection, coneDirection);
      if (spotResult > cos(light.coneAngle)) {
        float attenuation = 1.0 / (light.attenuation.x + light.attenuation.y * d + light.attenuation.z * d * d);
        attenuation *= pow(spotResult, light.coneAttenuation);
        float diffuseIntensity = saturate(dot(-lightDirection, normalDirection));
        float3 color = light.color * baseColor * diffuseIntensity; 
        color *= attenuation;
        diffuseColor += color;
      }
    }
  }

  float3 color = diffuseColor + ambientColor + specularColor;
  // float3 color = diffuseColor + ambientColor;
  return float4(color, 1);
}