
in vec3 pos;
in vec2 uvs;

layout (location = 0) out vec4 color;

#define M_PI 3.1415926535897932384626433832795

vec2 fun(vec2 z, vec2 c)
{
    return vec2(z.x * z.x - z.y * z.y, 2.0 * z.x * z.y) + c;
}

void main()
{

    float radius = 0.5;
    float x = uvs.x - 0.5;
    float y = uvs.y - 0.5;

    float d = sqrt(pow(x, 2.0) + pow(y, 2.0));

    if(d < radius) {
        color = vec4(1.0 - d, 1.0 - d, 1.0 - d, 1.0);
    } else {
        color = vec4(1.0, 1.0, 1.0, 0.0);
    }

//    float alpha = clamp(1.0 - abs(uvs.x - 0.5), 0.0, 1.0);
//
//    color = vec4(1.0, 1.0, 1.0, alpha);
}