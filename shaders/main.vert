#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 colors;

layout(location = 0) out vec4 vertexColor;

void main()
{
    vertexColor = vec4(colors, 1.0);
    gl_Position = vec4(position, 1.0);
}
