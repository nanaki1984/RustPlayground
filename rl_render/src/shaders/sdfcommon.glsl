struct SDFPrimitive {
    vec4 half_size_radius;
    vec4 inv_xform[3];

    float distance_scaling_factor;

    uint shape;
    uint group_id;

    //float soft_blend_radius;
    //uint blend_mode; // Default (Add), Subtract, Union, ...
};

float SDF_sphere_internal(vec3 position, float radius) {
    float d = max(max(position.x - radius, position.y), position.z);
    d = min(d, max(max(position.x, position.y - radius), position.z));
    d = min(d, max(max(position.x, position.y), position.z - radius));
    {
        float b = dot(vec2(1.0), position.yz);
        float c = dot(position.yz, position.yz) - (radius * radius);
        float discriminant = b * b - 2.0 * c;
        if (discriminant >= 0.0) {
            float t = (b - sqrt(discriminant)) / 2.0;
            d = min(d, max(t, position.x));
        }
    }
    {
        float b = dot(vec2(1.0), position.xz);
        float c = dot(position.xz, position.xz) - (radius * radius);
        float discriminant = b * b - 2.0 * c;
        if (discriminant >= 0.0) {
            float t = (b - sqrt(discriminant)) / 2.0;
            d = min(d, max(t, position.y));
       }
    }
    {
        float b = dot(vec2(1.0), position.xy);
        float c = dot(position.xy, position.xy) - (radius * radius);
        float discriminant = b * b - 2.0 * c;
        if (discriminant >= 0.0) {
            float t = (b - sqrt(discriminant)) / 2.0;
            d = min(d, max(t, position.z));
        }
    }

    float b = dot(vec3(1.0), position);
    float c = dot(position, position) - (radius * radius);
    float discriminant = b * b - 3.0 * c;
    if (discriminant >= 0.0) {
        float t = (b - sqrt(discriminant)) / 3.0;
        d = min(d, t);
    }

    return d;
}

float SDF_sphere(vec3 position, float radius) {
    return SDF_sphere_internal(abs(position), radius);
}

float SDF_box(vec3 position, vec3 half_size) {
    vec3 p = abs(position) - half_size;
    return max(p.x, max(p.y, p.z));
}

float SDF_rounded_box(vec3 position, vec3 half_size, float radius) {
    vec3 p = abs(position) - half_size;
    return SDF_sphere_internal(p, radius);
}

float SDF_soft_min(float a, float b, float r) {
    float e = max(r - abs(a - b), 0.0);
    return min(a, b) - e*e*0.25f/r;
}

float SDF_soft_max(float a, float b, float r) {
    float e = max(r - abs(a - b), 0.0);
    return max(a, b) + e*e*0.25f/r;
}

float SDF_primitive(SDFPrimitive primitive, vec3 world_position) {
    vec3 position = vec3(
        dot(vec4(world_position, 1.0), primitive.inv_xform[0]),
        dot(vec4(world_position, 1.0), primitive.inv_xform[1]),
        dot(vec4(world_position, 1.0), primitive.inv_xform[2]));

    float sdf_value = 0.0;
    switch (primitive.shape) {
        case 0: // Sphere
            sdf_value = SDF_sphere(position, primitive.half_size_radius.w);
            break;
        case 1: // Box
            sdf_value = SDF_box(position, primitive.half_size_radius.xyz);
            break;
        case 2: // Rounded Box
            sdf_value = SDF_rounded_box(position, primitive.half_size_radius.xyz, primitive.half_size_radius.w);
            break;
    }

    return primitive.distance_scaling_factor * sdf_value;
}
