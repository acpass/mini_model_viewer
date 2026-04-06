#version 450

layout(location = 0) in vec3 aColor;
layout(location = 0) out vec4 vColor;

void main()
{
    vColor = vec4(aColor, 1.0);
}
