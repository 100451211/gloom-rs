#version 430 core

//out vec4 color;

layout(location=2) in vec4 in_color;
out vec4 color;


void main()
{
    // ** LAB 1 
    // Faded color in triangles using gl_FragCoord
    color = vec4(gl_FragCoord.x / 650.0f, gl_FragCoord.y / 650.0f, 0.5f, 1.0f);

    // color = vec4(1.0f, 1.0f, 1.0f, 1.0f); // WHITE
     
    // color = vec4(0.0f, 1.0f, 0.0f, 1.0f); // GREEN

    // **LAB 2 - 1b
    color = in_color;

}