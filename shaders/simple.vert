#version 430 core

//in vec3 position;

layout(location=0) in vec3 position;

layout(location=2) in vec4 in_color;
out vec4 frag_color;

uniform (location=3) mat4x4 composite_matrix;

void main()
{
    /*mat4x4 identity_matrix = {
        { 1, 0, 0, 0 },
        { 0, 1, 0, 0 },
        { 0, 0, 1, 0 },
        { 0, 0, 0, 1 },
    };
    gl_Position = identity_matrix * vec4(position, 1.0f);*/

    float a = 0;
    float b = 0;
    float c = 0;
    float d = 0;
    float e = 0;
    float f = 1;

    mat4x4 identity_matrix = {
        { a+1, d+0, 0, 0 },
        { b+0, e+1, 0, 0 },
        { 0, 0, 1, 0 },
        { c+0, f+0, 0, 1 },
    };
    gl_Position = identity_matrix * vec4(position, 1.0f);

    gl_Position =  affine_transformation_matrix * vec4(position, 1.0f);

    //gl_Position = vec4(position, 1.0f);
    //gl_Position = vec4(-position, 1.0f); // con esto tambien se gira todo
    //gl_Position = vec4(-position.x, -position.y, position.z, 1.0f);   //con esto giro todo
    frag_color = in_color;
}