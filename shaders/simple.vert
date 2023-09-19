
#version 430 core

in vec3 position;

//layout(location=1) vec3 position;

layout(location=2) in vec4 in_color;
layout(location=2) out vec4 out_color;

void main()
{
    // * LAB 1
    // gl_Position = vec4(position, 1.0f); // ORIGINAL
    // gl_Position = vec4(-position.x, -position.y, position.z, 1.0f); // FLIPPED

    // * LAB 2 - 1b
    gl_Position = vec4(position, 1.0f);
    out_color = in_color;
    
}