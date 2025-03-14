@group(0) @binding(0) var my_texture: texture_2d<f32>;
@group(0) @binding(1) var my_sampler: sampler;

@fragment
fn fs_main(@builtin(position) frag_coord: vec2<f32>) -> @location(0) vec4<f32> {
    let tex_size = vec2<f32>(textureDimensions(my_texture));
    let uv = frag_coord / tex_size;

    let offset = vec2<f32>(1.0 / tex_size.x, 1.0 / tex_size.y);
    let color = textureSample(my_texture, my_sampler, uv) * 0.4 +
                textureSample(my_texture, my_sampler, uv + offset) * 0.15 +
                textureSample(my_texture, my_sampler, uv - offset) * 0.15 +
                textureSample(my_texture, my_sampler, uv + vec2<f32>(offset.x, -offset.y)) * 0.15 +
                textureSample(my_texture, my_sampler, uv + vec2<f32>(-offset.x, offset.y)) * 0.15;

    return color;
}
