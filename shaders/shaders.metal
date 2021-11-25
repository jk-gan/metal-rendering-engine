#include <metal_stdlib>

using namespace metal;

struct VertexOut {
  float4 position [[position]];
  float4 color;
};

vertex VertexOut vertex_main(uint vid [[vertex_id]],
             constant float4 *position [[buffer(0)]],
             constant float4 *color [[buffer(1)]]) {
  VertexOut out {
    .position = position[vid],
    .color = color[vid]
  };
  return out;
}

fragment float4 fragment_main(VertexOut vertex_out [[ stage_in ]]) {
  return vertex_out.color;
}
