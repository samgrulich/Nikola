struct Color {
    @location(0) rgb: vec3<f32>,
    @location(1) alpha: f32,
};

struct Sphere {
    center: vec3<f32>,
    radius: f32,
};

@group(0) @binding(0) var out_texture: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(1) var<storage> input_color: Color;

@compute @workgroup_size(8, 8)
fn main(
    @builtin(global_invocation_id) global_id: vec3<u32>
) {
    let dimensions = textureDimensions(out_texture);
    let pixel_coords = vec2<i32>(global_id.xy);

    if (pixel_coords.x >= dimensions.x || pixel_coords.y >= dimensions.y) {
        return;
    }
   
    let dims = vec2<f32>(dimensions);
    let ratio = dimensions.x / dimensions.y;

    var coords = (vec2<f32>(global_id.xy) - dims / 2f) / dims; 
    coords.x *= f32(ratio);
    coords.y *= -1f;

    let fov = 90f; 
    let cam_pos = vec3(0f);
    let cam_o = vec3(0f, 0f, tan(fov / 2f));

    let ray_o = vec3(coords, 0f);
    let ray_d = normalize(ray_o - cam_o);

    let sphere_o = vec3(0f, 0f, -10f) - cam_pos;
    let sphere_r = 1f;

    let center = ray_o - sphere_o;
    
    let a = dot(ray_d, ray_d);            
    let b = 2f * dot(ray_d, center);
    let c = dot(center, center) - pow(sphere_r, 2f);

    let D = pow(b, 2f) - 4f * a * c;
    
    var color = vec3(0.01f);

    if (D > 0f)
    {
        let t1 = (-b + sqrt(D)) / 2f * a;
        let t = (-b - sqrt(D)) / (2f * a);

        if (t > 0f)
        {
            let P = ray_d * t + ray_o;
  
            let light = normalize(vec3(1f, 1f, 1f));
            let normal = normalize(P - sphere_o);
       
            let albedo = vec3<f32>(1f, 0f, 1f);
            color = albedo * max(dot(normal, light), 0f);
        }
    }

    let position = vec3<f32>(global_id);
    let color = vec4<f32>(color, input_color.alpha);
    textureStore(out_texture, pixel_coords.xy, color);
}
