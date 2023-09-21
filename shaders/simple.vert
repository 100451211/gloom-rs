#version 430 core

//in vec3 position;

layout(location=0) in vec3 position;

layout(location=2) in vec4 in_color;
out vec4 frag_color;

void main()
{
    gl_Position = vec4(position, 1.0f);
    //gl_Position = vec4(-position, 1.0f); // con esto tambien se gira todo
    //gl_Position = vec4(-position.x, -position.y, position.z, 1.0f);   //con esto giro todo
    frag_color = in_color;
}