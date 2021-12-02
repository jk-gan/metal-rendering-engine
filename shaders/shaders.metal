#include <metal_stdlib>
#import "../shader_types/shader_types.h"

using namespace metal;

struct VertexIn {
  float4 position [[attribute(0)]];
  float3 normal [[attribute(1)]];
};

struct VertexOut {
  float4 position [[position]];
  float3 normal;
};

// struct VertexOut {
//   float4 position [[position]];
//   float4 color;
// };

vertex VertexOut vertex_main(VertexIn vertexIn [[stage_in]], constant Uniforms &uniforms [[buffer(2)]]) {
  VertexOut out {
    .position = uniforms.projectionMatrix * uniforms.viewMatrix * uniforms.modelMatrix * vertexIn.position,
    .normal = vertexIn.normal
  };
  return out;
}

fragment float4 fragment_main(VertexOut in [[stage_in]]) {
  // return float4(1, 1, 1, 0.5);
  return float4(in.normal, 1);
}

// vertex VertexOut vertex_main(uint vid [[vertex_id]],
//              constant float4 *position [[buffer(0)]],
//              constant float4 *color [[buffer(1)]]) {
//   VertexOut out {
//     .position = position[vid],
//     .color = color[vid]
//   };
//   return out;
// }

// fragment float4 fragment_main(VertexOut vertex_out [[ stage_in ]]) {
//   return vertex_out.color;
// }