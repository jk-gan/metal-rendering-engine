#ifndef shader_types_h
#define shader_types_h

#include <simd/simd.h>

typedef struct {
  matrix_float4x4 modelMatrix;
  matrix_float4x4 viewMatrix;
  matrix_float4x4 projectionMatrix;
  matrix_float3x3 normalMatrix;
} Uniforms;

typedef enum {
  unused = 0,
  Sunlight = 1,
  Spotlight = 2,
  Pointlight = 3,
  Ambientlight = 4
} LightType;

typedef struct {
  vector_float3 position;
  vector_float3 color;
  vector_float3 specularColor;
  float intensity;
  vector_float3 attenuation;
  LightType type;
  float coneAngle;
  vector_float3 coneDirection;
  float coneAttenuation;
} Light;

typedef struct {
  unsigned int lightCount;
  vector_float3 cameraPosition;
  unsigned int tiling;
} FragmentUniforms;

typedef enum {
  BufferIndexVertices = 0,
  BufferIndexLights = 1,
  BufferIndexUniforms = 2,
  BufferIndexFragmentUniforms = 3,
  BufferIndexSkybox = 13,
  BufferIndexMaterials = 14
} BufferIndices;

typedef enum {
  Position = 0,
  Normal = 1,
  UV = 2,
  Tangent = 3,
  Bitangent = 4
} Attributes;

typedef enum {
  BaseColorTexture = 0,
  NormalTexture = 1,
  MetallicRoughnessTexture = 2,
  OcclusionTexture = 3,
  EmissiveTexture = 4,
  CubeMap = 5
} Textures;

typedef struct {
  vector_float4 baseColor;
  vector_float4 specularColor;
  float roughness;
  float metallic;
  // vector_float3 ambientOcclusion;
  float shininess;
} Material;

#endif /* shader_types.h */
