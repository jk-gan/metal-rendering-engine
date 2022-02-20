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
  float3 reflectedVector;
  float3 reflectedColor;
  float3 irradiatedColor;
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

fragment float4 skybox_test(VertexOut in [[stage_in]],
          constant Light *lights [[buffer(BufferIndexLights)]],
          constant Material &material [[buffer(BufferIndexMaterials)]],
          sampler textureSampler [[sampler(0)]],
          constant FragmentUniforms &fragmentUniforms [[buffer(BufferIndexFragmentUniforms)]],
          texturecube<float> skybox [[texture(CubeMap)]]) {
  float3 viewDirection = in.worldPosition.xyz - fragmentUniforms.cameraPosition;
  float3 textureCoordinates = reflect(viewDirection, in.worldNormal);

  constexpr sampler defaultSampler(filter::linear);
  float4 color = skybox.sample(defaultSampler, textureCoordinates);
  float4 copper = float4(0.86, 0.7, 0.48, 1);
  color = color * copper;
  return color;
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
          texture2d<float> emissiveTexture [[texture(EmissiveTexture), function_constant(hasEmissiveTexture)]],
          texturecube<float> skybox [[texture(CubeMap)]],
          texturecube<float> skyboxDiffuse [[texture(CubeMapDiffuse)]],
          texture2d<float> brdfLut [[texture(BRDFLut)]]) {
  // extract color
  float3 baseColor;
  if (hasColorTexture) {
    baseColor = pow(baseColorTexture.sample(textureSampler,
                                        in.uv * fragmentUniforms.tiling).rgb, 2.2);
  } else {
    baseColor = material.baseColor.rgb;
  }
  // extract metallic and roughness
  float metallic;
  float roughness;
  if (hasMetallicRoughnessTexture) {
    metallic = metallicRoughnessTexture.sample(textureSampler, in.uv).r;
    roughness = metallicRoughnessTexture.sample(textureSampler, in.uv).g;
    /* metallic = metallicRoughnessTexture.sample(textureSampler, in.uv).r * material.metallic; */
    /* roughness = metallicRoughnessTexture.sample(textureSampler, in.uv).g * material.roughness; */
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
    /* emissiveColor = emissiveTexture.sample(textureSampler, in.uv).rgb * float3(1, 1, 1); */
    emissiveColor = emissiveTexture.sample(textureSampler, in.uv).rgb;
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

  float4 diffuse = skyboxDiffuse.sample(textureSampler, normal);
  diffuse = mix(pow(diffuse, 0.5), diffuse, metallic);

  /* float3 viewDirection = in.worldPosition.xyz - fragmentUniforms.cameraPosition; */
  float3 viewDirection = normalize(fragmentUniforms.cameraPosition - in.worldPosition);
  float3 textureCoordinates = reflect(-viewDirection, normal);

  constexpr sampler s(filter::linear, mip_filter::linear);
  float3 prefilteredColor = skybox.sample(s, textureCoordinates,
                                          level(roughness * 10)).rgb;

  float nDotV = saturate(dot(normal, normalize(-viewDirection)));
  float2 envBRDF = brdfLut.sample(s, float2(roughness, nDotV)).rg;

  float3 f0 = mix(0.04, baseColor.rgb, metallic);
  float3 specularIBL = f0 * envBRDF.r + envBRDF.g;
  
  float3 specular = prefilteredColor * specularIBL;
  float4 color = diffuse * float4(baseColor, 1) + float4(specular, 1);
  color *= ambientOcclusion;
  color += float4(emissiveColor, 1.0);

  return color;
  
  // float3 viewDirection = normalize(fragmentUniforms.cameraPosition - in.worldPosition);
  
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
  lighting.irradiatedColor = prefilteredColor;
  
  float3 specularOutput = render(lighting);
  
  // compute Lambertian diffuse
  float nDotl = max(0.001, saturate(dot(lighting.normal, lighting.lightDirection)));
  float3 diffuseColor = float3(((1.0/pi) * baseColor) * (1.0 - metallic));
  diffuseColor = diffuseColor * nDotl * ambientOcclusion;
  /* float3 diffuseColor = diffuse * baseColor * nDotl * ambientOcclusion; */
  /* diffuseColor *= 1.0 - metallic; */
  
  float4 finalColor = float4(specularOutput, 1.0) + (diffuse * float4(diffuseColor, 1.0)) + float4(emissiveColor, 1.0);
  return finalColor;
}

/* // GGX distribution */
/* float D_GGX(float NoH, float roughness) { */
/*   float a = NoH * roughness; */
/*   float k = roughness / (1.0 - NoH * NoH + a * a); */
/*   return k * k * (1.0 / PI); */
/* } */

/* float optimized_D_GGX(float roughness, float NoH, const float3 n, const float3 h) { */
/*   float MEDIUMP_FLT_MAX = 65504.0; */

/*   float3 NxH = cross(n, h); */
/*   float a = NoH * roughness; */
/*   float k = roughness / (dot(NxH, Nxh) + a * a); */
/*   float d = k * k * (1.0 / PI); */

/*   return min(d, MEDIUMP_FLT_MAX); */
/* } */

/* // Geometric shadowing */
/* float V_SmithGGXCorrelatedFast(float NoV, float NoL, float roughness) { */
/*   float a = roughness; */
/*   float GGXV = NoL * (NoV * (1.0 - a) + a); */
/*   float GGXL = NoV * (NoL * (1.0 - a) + a); */
/*   return 0.5 / (GGXV + GGXL); */
/* } */

/* // Fresnel */
/* float3 F_Schlick(float u, float3 f0, float f90) { */
/*   return f0 + (float3(f90) - f0) * pow(1.0 - u, 5.0); */
/* } */

/* float3 optimized_F_Schlick(float u, float3 f0) { */
/*   float f = pow(1.0 - u, 5.0); */
/*   return f + f0 * (1.0 - f); */
/* } */

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
  } else {
    /* float a = nDoth * specularRoughness; */
    /* float k = specularRoughness / (1.0 - nDoth * nDoth + a * a); */
    /* Ds = k * k * (1.0 / pi); */

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
  
  float3 specularOutput = (Ds * Gs * Fs * lighting.irradiatedColor) * (1.0 + lighting.metallic * lighting.baseColor) + lighting.metallic * lighting.irradiatedColor * lighting.baseColor;
  specularOutput = specularOutput * lighting.ambientOcclusion;
  
  return specularOutput;
}


