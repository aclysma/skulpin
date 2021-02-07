#include <metal_stdlib>
#include <simd/simd.h>

using namespace metal;

struct spvDescriptorSetBuffer0
{
    texture2d<float> texSampler [[id(0)]];
};

struct main0_out
{
    float4 outColor [[color(0)]];
};

struct main0_in
{
    float2 fragTexCoord [[user(locn0)]];
};

fragment main0_out main0(main0_in in [[stage_in]], constant spvDescriptorSetBuffer0& spvDescriptorSet0 [[buffer(0)]])
{
    constexpr sampler texSamplerSmplr(filter::linear, mip_filter::linear, address::repeat, compare_func::never, max_anisotropy(1));
    main0_out out = {};
    out.outColor = spvDescriptorSet0.texSampler.sample(spvDescriptorSet0.texSamplerSmplr, in.fragTexCoord);
    return out;
}

