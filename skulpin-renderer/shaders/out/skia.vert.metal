#include <metal_stdlib>
#include <simd/simd.h>

using namespace metal;

struct main0_out
{
    float2 fragTexCoord [[user(locn0)]];
    float4 gl_Position [[position]];
};

struct main0_in
{
    float2 inPosition [[attribute(0)]];
    float2 inTexCoord [[attribute(1)]];
};

vertex main0_out main0(main0_in in [[stage_in]])
{
    main0_out out = {};
    out.gl_Position = float4(in.inPosition, 0.0, 1.0);
    out.fragTexCoord = in.inTexCoord;
    return out;
}

