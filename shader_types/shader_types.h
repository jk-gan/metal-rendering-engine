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
} FragementUniforms;

typedef enum {
  BufferIndexVertices = 0,
  BufferIndexLights = 1,
  BufferIndexUniforms = 3,
  BufferIndexFragmentUniforms = 4
} BufferIndices;

typedef enum {
  Position = 0,
  Normal = 1,
  UV = 2
} Attributes;

typedef enum {
  BaseColorTexture = 0,
  NormalTexture = 1
} Textures;

#endif /* shader_types.h */
