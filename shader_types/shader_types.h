#ifndef shader_types_h
#define shader_types_h

#include <simd/simd.h>

typedef struct {
    vector_float2 position;
    vector_float2 texture_coord;
} TexuredVertex;

#endif /* shader_types.h */