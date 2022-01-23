#include <metal_stdlib>
#import "../../shader_types/shader_types.h"

using namespace metal;

constant bool hasColorTexture [[function_constant(0)]];
constant bool hasNormalTexture [[function_constant(1)]];
constant bool hasMetallicRoughnessTexture [[function_constant(2)]];
/* constant bool hasMetallicTexture [[function_constant(3)]]; */
constant bool hasAOTexture [[function_constant(3)]];
constant bool hasEmissiveTexture [[function_constant(4)]];

constant float pi = 3.1415926535897932384626433832795;

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
  float3 worldTangent;
  float3 worldBitangent;
  float2 uv;
};

typedef struct Lighting {
  float3 lightDirection;
  float3 viewDirection;
  float3 baseColor;
  float3 normal;
  float metallic;
  float roughness;
  float ambientOcclusion;
  float3 lightColor;
} Lighting;

float3 render (Lighting lighting);

vertex VertexOut vertex_main(VertexIn vertexIn [[stage_in]], constant Uniforms &uniforms [[buffer(BufferIndexUniforms)]]) {
  VertexOut out {
    .position = uniforms.projectionMatrix * uniforms.viewMatrix * uniforms.modelMatrix * vertexIn.position,
    .worldPosition = (uniforms.modelMatrix * vertexIn.position).xyz,
    .worldNormal = uniforms.normalMatrix * vertexIn.normal,
    .worldTangent = uniforms.normalMatrix * vertexIn.tangent,
    .worldBitangent = uniforms.normalMatrix * vertexIn.bitangent,
    .uv = vertexIn.uv,
  };
  return out;
}

fragment float4 fragment_main(VertexOut in [[stage_in]],
          constant Light *lights [[buffer(BufferIndexLights)]],
          constant Material &material [[buffer(BufferIndexMaterials)]],
          sampler textureSampler [[sampler(0)]],
          constant FragmentUniforms &fragmentUniforms [[buffer(BufferIndexFragmentUniforms)]],
          texture2d<float> baseColorTexture [[texture(BaseColorTexture), function_constant(hasColorTexture)]],
          texture2d<float> normalTexture [[texture(NormalTexture), function_constant(hasNormalTexture)]],
          texture2d<float> metallicRoughnessTexture [[texture(MetallicRoughnessTexture), function_constant(hasMetallicRoughnessTexture)]],
          texture2d<float> aoTexture [[texture(OcclusionTexture), function_constant(hasAOTexture)]],
          texture2d<float> emissiveTexture [[texture(EmissiveTexture), function_constant(hasEmissiveTexture)]]) {
  // extract color
  float3 baseColor;
  if (hasColorTexture) {
    baseColor = baseColorTexture.sample(textureSampler,
                                        in.uv * fragmentUniforms.tiling).rgb;
  } else {
    baseColor = material.baseColor.rgb;
  }
  // extract metallic and roughness
  float metallic;
  float roughness;
  if (hasMetallicRoughnessTexture) {
    metallic = metallicRoughnessTexture.sample(textureSampler, in.uv).r * material.metallic;
    roughness = metallicRoughnessTexture.sample(textureSampler, in.uv).g * material.roughness;
  } else {
    metallic = material.metallic;
    roughness = material.roughness;
  }
  // extract ambient occlusion
  float ambientOcclusion;
  if (hasAOTexture) {
    ambientOcclusion = aoTexture.sample(textureSampler, in.uv).r;
  } else {
    ambientOcclusion = 0.0;
  }

  float3 emissiveColor;
  if (hasEmissiveTexture) {
    emissiveColor = emissiveTexture.sample(textureSampler, in.uv).rgb * float3(1, 1, 1);
  } else {
    emissiveColor = float3(0, 0, 0);
  }
  
  // normal map
  float3 normal;
  if (hasNormalTexture) {
    float3 normalValue = normalTexture.sample(textureSampler, in.uv * fragmentUniforms.tiling).xyz * 2.0 - 1.0;
    normal = in.worldNormal * normalValue.z
    + in.worldTangent * normalValue.x
    + in.worldBitangent * normalValue.y;
  } else {
    normal = in.worldNormal;
  }
  normal = normalize(normal);
  
  float3 viewDirection = normalize(fragmentUniforms.cameraPosition - in.worldPosition);
  
  Light light = lights[0];
  float3 lightDirection = normalize(light.position);
  lightDirection = light.position;
  
  // all the necessary components are in place
  Lighting lighting;
  lighting.lightDirection = lightDirection;
  lighting.viewDirection = viewDirection;
  lighting.baseColor = baseColor;
  lighting.normal = normal;
  lighting.metallic = metallic;
  lighting.roughness = roughness;
  lighting.ambientOcclusion = ambientOcclusion;
  lighting.lightColor = light.color;
  
  float3 specularOutput = render(lighting);
  
  // compute Lambertian diffuse
  float nDotl = max(0.001, saturate(dot(lighting.normal, lighting.lightDirection)));
  float3 diffuseColor = light.color * baseColor * nDotl * ambientOcclusion;
  diffuseColor *= 1.0 - metallic;
  
  float4 finalColor = float4(specularOutput + diffuseColor + emissiveColor, 1.0);
  return finalColor;
}

/*
PBR.metal rendering equation from Apple's LODwithFunctionSpecialization sample code is under Copyright © 2017 Apple Inc.

Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
*/


float3 render(Lighting lighting) {
  // Rendering equation courtesy of Apple et al.
  float nDotl = max(0.001, saturate(dot(lighting.normal, lighting.lightDirection)));
  float3 halfVector = normalize(lighting.lightDirection + lighting.viewDirection);
  float nDoth = max(0.001, saturate(dot(lighting.normal, halfVector)));
  float nDotv = max(0.001, saturate(dot(lighting.normal, lighting.viewDirection)));
  float hDotl = max(0.001, saturate(dot(lighting.lightDirection, halfVector)));
  
  // specular roughness
  float specularRoughness = lighting.roughness * (1.0 - lighting.metallic) + lighting.metallic;
  
  // Normal Distribution Function
  float Ds;
  if (specularRoughness >= 1.0) {
    Ds = 1.0 / pi;
  }
  else {
    float roughnessSqr = specularRoughness * specularRoughness;
    float d = (nDoth * roughnessSqr - nDoth) * nDoth + 1;
    Ds = roughnessSqr / (pi * d * d);
  }
  
  // Fresnel
  float3 Cspec0 = float3(1.0);
  float fresnel = pow(clamp(1.0 - hDotl, 0.0, 1.0), 5.0);
  float3 Fs = float3(mix(float3(Cspec0), float3(1), fresnel));
  
  // Geometry
  float alphaG = (specularRoughness * 0.5 + 0.5) * (specularRoughness * 0.5 + 0.5);
  float a = alphaG * alphaG;
  float b1 = nDotl * nDotl;
  float b2 = nDotv * nDotv;
  float G1 = (float)(1.0 / (b1 + sqrt(a + b1 - a*b1)));
  float G2 = (float)(1.0 / (b2 + sqrt(a + b2 - a*b2)));
  float Gs = G1 * G2;
  
  float3 specularOutput = (Ds * Gs * Fs * lighting.lightColor) * (1.0 + lighting.metallic * lighting.baseColor) + lighting.metallic * lighting.lightColor * lighting.baseColor;
  specularOutput = specularOutput * lighting.ambientOcclusion;
  
  return specularOutput;
}


