#version 430 core

in vec4 frag_color;
//layout(location=2) in vec4 in_color;

out vec4 color;

void main()
{
    /*float x = gl_FragCoord.x / 800; // Esto es lo que hace que cambie el color al dividir entre 800
    float y = gl_FragCoord.y / 800;
    
    color = vec4(1.0 - x, 1.0 - y, 1.0, 1.0);*/ //con esto pongo color en funcion del punto en el que el pixel se encuentre een el lienzo
    //color = vec4(gl_FragCoord.x / 650.0f, gl_FragCoord.y / 650.0f, 0.5f, 1.0f);

    //color = vec4(1.0f, 1.0f, 1.0f, 1.0f); // blanco

    //color = vec4(1.0f, 0.0f, 0.0f, 1.0f); // rojo

    //color = vec4(0.0f, 1.0f, 0.0f, 1.0f); // verde

    //color = vec4(0.0f, 0.0f, 1.0f, 1.0f); // azul

    //color = vec4(0.0f, 0.0f, 0.0f, 1.0f); // negro

    //color = vec4(0.0f, 0.0f, 0.0f, 0.0f); // figura no aparece
    color = frag_color;
}