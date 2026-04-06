#version 450

vec3 triangle[3] = vec3[](
        vec3(0.0, 0.5, 0.0),
        vec3(-0.5, -0.5, 0.0),
        vec3(0.5, -0.5, 0.0)
    );

vec3 colors[3] = vec3[](
        vec3(1.0, 0.0, 0.0),
        vec3(0.0, 1.0, 0.0),
        vec3(0.0, 0.0, 1.0)
    );

layout(location = 0) out vec4 vertexColor;

void main()
{
    vertexColor = vec4(colors[gl_VertexIndex], 1.0);
    gl_Position = vec4(triangle[gl_VertexIndex], 1.0);
}
