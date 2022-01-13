#include <metal_stdlib>
#import "../shader_types/shader_types.h"

using namespace metal;

constant bool hasColorTexture [[function_constant(0)]];
constant bool hasNormalTexture [[function_constant(1)]];

struct VertexIn {
  float4 position [[attribute(Position)]];
  float3 normal [[attribute(Normal)]];
  float2 uv [[attribute(UV)]];
  float3 tangent [[attribute(Tangent)]];
  float3 bitangent [[attribute(Bitangent)]];
};

struct VertexOut {
  float4 position [[position]];
  float3 worldPosition;
  float3 worldNormal;
  float2 uv;
  float3 worldTangent;
  float3 worldBitangent;
};

vertex VertexOut vertex_main(VertexIn vertexIn [[stage_in]], constant Uniforms &uniforms [[buffer(BufferIndexUniforms)]]) {
  VertexOut out {
    .position = uniforms.projectionMatrix * uniforms.viewMatrix * uniforms.modelMatrix * vertexIn.position,
    .worldPosition = (uniforms.modelMatrix * vertexIn.position).xyz,
    .worldNormal = uniforms.normalMatrix * vertexIn.normal,
    .uv = vertexIn.uv,
    .worldTangent = uniforms.normalMatrix * vertexIn.tangent,
    .worldBitangent = uniforms.normalMatrix * vertexIn.bitangent
  };
  return out;
}

fragment float4 fragment_main(VertexOut in [[stage_in]],
  constant Material &material [[buffer(BufferIndexMaterials)]],
  texture2d<float> baseColorTexture [[texture(BaseColorTexture), function_constant(hasColorTexture)]],
  texture2d<float> normalTexture [[texture(NormalTexture), function_constant(hasNormalTexture)]],
  sampler textureSampler [[sampler(0)]], 
  constant Light *lights [[buffer(BufferIndexLights)]],
  constant FragmentUniforms &fragmentUniforms [[buffer(BufferIndexFragmentUniforms)]]) {

  // float3 baseColor = baseColorTexture.sample(textureSampler, in.uv * fragmentUniforms.tiling).rgb;
  float3 baseColor;
  if (hasColorTexture) {
    baseColor = baseColorTexture.sample(textureSampler, in.uv * fragmentUniforms.tiling).rgb;
  } else {
    baseColor = material.baseColor;
  }
  float3 normalValue;
  if (hasNormalTexture) {
    normalValue = normalTexture.sample(textureSampler, in.uv * fragmentUniforms.tiling).xyz;
    normalValue = normalValue * 2 - 1;
  } else {
    normalValue = in.worldNormal;
  }
  normalValue = normalize(normalValue);

  float3 diffuseColor = 0;
  float3 ambientColor = 0;
  float3 specularColor = 0;
  // float materialShininess = 64;
  // float3 materialSpecularColor = float3(0.4, 0.4, 0.4);
  float3 materialSpecularColor = material.specularColor;
  float materialShininess = material.shininess;

  // float3 normalDirection = normalize(in.worldNormal);
  float3 normalDirection = float3x3(in.worldTangent, in.worldBitangent, in.worldNormal) * normalValue;
  normalDirection = normalize(normalDirection);

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

  float3 color = saturate(diffuseColor + ambientColor + specularColor);
  // float3 color = diffuseColor + ambientColor;
  return float4(color, 1);
}
