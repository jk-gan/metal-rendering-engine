#include <metal_stdlib>
#import "../shader_types/shader_types.h"

using namespace metal;

struct VertexIn {
  float4 position [[attribute(0)]];
};

struct VertexOut {
  float4 position [[position]];
  float point_size [[point_size]];
};

// struct VertexOut {
//   float4 position [[position]];
//   float4 color;
// };

vertex float4 vertex_main(VertexIn vertexIn [[stage_in]], constant Uniforms &uniforms [[buffer(1)]]) {
  // VertexOut out {
  //   .position = float4(in.position, 1)
  // };
  // return out;
  float4 position = uniforms.projectionMatrix * uniforms.viewMatrix * uniforms.modelMatrix * vertexIn.position;
  return position;
}

fragment float4 fragment_main(VertexOut vertex_out [[stage_in]]) {
  return float4(1, 1, 1, 0.5);
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