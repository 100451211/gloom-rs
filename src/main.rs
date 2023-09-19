// Uncomment these following global attributes to silence most warnings of "low" interest:

#![allow(dead_code)]
#![allow(non_snake_case)]
#![allow(unreachable_code)]
#![allow(unused_mut)]
#![allow(unused_unsafe)]
#![allow(unused_variables)]

extern crate nalgebra_glm as glm;
use std::{ mem, ptr, os::raw::c_void };
use std::thread;
use std::sync::{Mutex, Arc, RwLock};

mod shader;
mod util;

use glutin::event::{Event, WindowEvent, DeviceEvent, KeyboardInput, ElementState::{Pressed, Released}, VirtualKeyCode::{self, *}};
use glutin::event_loop::ControlFlow;

// initial window size
const INITIAL_SCREEN_W: u32 = 800;
const INITIAL_SCREEN_H: u32 = 600;

// == // Helper functions to make interacting with OpenGL a little bit prettier. You *WILL* need these! // == //

// Get the size of an arbitrary array of numbers measured in bytes
// Example usage:  pointer_to_array(my_array)
fn byte_size_of_array<T>(val: &[T]) -> isize {
    std::mem::size_of_val(&val[..]) as isize
}

// Get the OpenGL-compatible pointer to an arbitrary array of numbers
// Example usage:  pointer_to_array(my_array)
fn pointer_to_array<T>(val: &[T]) -> *const c_void {
    &val[0] as *const T as *const c_void
}

// Get the size of the given type in bytes
// Example usage:  size_of::<u64>()
fn size_of<T>() -> i32 {
    mem::size_of::<T>() as i32
}

// Get an offset in bytes for n units of type T, represented as a relative pointer
// Example usage:  offset::<u64>(4)
fn offset<T>(n: u32) -> *const c_void {
    (n * mem::size_of::<T>() as u32) as *const T as *const c_void
}

// Get a null pointer (equivalent to an offset of 0)
// ptr::null()

// == // Generate your VAO here
// Message for git commit
unsafe fn create_vao(vertices: &Vec<f32>, indices: &Vec<u32>, colors: &Vec<f32>) -> u32 {
    // Implement me!

    // Also, feel free to delete comments :)

    // Create VAO 
    let mut vao = 0;

    // * Generate a VAO and bind it
    gl::GenVertexArrays(5, &mut vao);
    assert!(vao != 0);
    gl::BindVertexArray(vao);
    
    // * Generate a VBO and bind it
    let mut vbo = 0;
    gl::GenBuffers(1,&mut vbo );
    gl::BindBuffer(gl::ARRAY_BUFFER , vbo);

    // * Fill it with data
    gl::BufferData(gl::ARRAY_BUFFER, 
        byte_size_of_array(vertices), 
        pointer_to_array(vertices), 
        gl::STATIC_DRAW);
    
    // * Configure a VAO for the data and enable it
    let index:u32 = 0;
    gl::VertexAttribPointer(index, 
        3,
        gl::FLOAT,
        gl::FALSE,
        0,
        ptr::null());
    gl::EnableVertexAttribArray(index);

    // * Generate a IBO and bind it
    let mut ibo = 0;
    gl::GenBuffers(1, &mut ibo);
    gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ibo);

    // * Fill it with data
    gl::BufferData(gl::ELEMENT_ARRAY_BUFFER, 
        byte_size_of_array(indices),
        pointer_to_array(indices), 
        gl::STATIC_DRAW
    );


    // ** LAB 2 -  1a **

    let mut cbo_id: u32 = 0;
    gl::GenBuffers(1, &mut cbo_id);
    gl::BindBuffer(gl::ARRAY_BUFFER, cbo_id);
    gl::BufferData(
        gl::ARRAY_BUFFER,
        (colors.len() * std::mem::size_of::<f32>()) as isize,
        colors.as_ptr() as *const std::ffi::c_void,
        gl::STATIC_DRAW,
    );

    let color_attrbute_index: u32 = 1;
    let nr_of_color_values = 4;
    gl::VertexAttribPointer(
        color_attrbute_index,                        // Index
        nr_of_color_values,                      // Size
        gl::FLOAT,                              // Type
        gl::FALSE,                         // Normalized
        size_of::<f32>() * nr_of_color_values, // Stride
        offset::<c_void>(0),                  // Offset
    );
    gl::EnableVertexAttribArray(color_attrbute_index);

    // * Create color vector with 1 value per vertex and bind it
    // let mut colors = vec![1.0; vertices.len()];
    // let mut c_buf = 0;
    // gl::GenBuffers(1, &mut c_buf);
    // gl::BindBuffer(gl::ARRAY_BUFFER, c_buf);


    // // * Fill it with data
    // gl::BufferData(gl::ARRAY_BUFFER, 
    //     byte_size_of_array(&colors),
    //     pointer_to_array(&colors), 
    //     gl::STATIC_DRAW
    // );

    // // * put into a Vertex Buffer Object, and attached to the VAO
    // gl::VertexAttribPointer(
    //     1, 
    //     4,
    //     gl::FLOAT,
    //     gl::FALSE,
    //     byte_size_of_array(&colors),
    //     ptr::null()
    // );
    // gl::EnableVertexAttribArray(1);


    // * Return the ID of the VAO
    return vao;

}


fn main() {
    // Set up the necessary objects to deal with windows and event handling
    let el = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new()
        .with_title("Gloom-rs")
        .with_resizable(true)
        .with_inner_size(glutin::dpi::LogicalSize::new(INITIAL_SCREEN_W, INITIAL_SCREEN_H));
    let cb = glutin::ContextBuilder::new()
        .with_vsync(true);
    let windowed_context = cb.build_windowed(wb, &el).unwrap();
    // Uncomment these if you want to use the mouse for controls, but want it to be confined to the screen and/or invisible.
    // windowed_context.window().set_cursor_grab(true).expect("failed to grab cursor");
    // windowed_context.window().set_cursor_visible(false);

    // Set up a shared vector for keeping track of currently pressed keys
    let arc_pressed_keys = Arc::new(Mutex::new(Vec::<VirtualKeyCode>::with_capacity(10)));
    // Make a reference of this vector to send to the render thread
    let pressed_keys = Arc::clone(&arc_pressed_keys);

    // Set up shared tuple for tracking mouse movement between frames
    let arc_mouse_delta = Arc::new(Mutex::new((0f32, 0f32)));
    // Make a reference of this tuple to send to the render thread
    let mouse_delta = Arc::clone(&arc_mouse_delta);

    // Set up shared tuple for tracking changes to the window size
    let arc_window_size = Arc::new(Mutex::new((INITIAL_SCREEN_W, INITIAL_SCREEN_H, false)));
    // Make a reference of this tuple to send to the render thread
    let window_size = Arc::clone(&arc_window_size);

    // Spawn a separate thread for rendering, so event handling doesn't block rendering
    let render_thread = thread::spawn(move || {
        // Acquire the OpenGL Context and load the function pointers.
        // This has to be done inside of the rendering thread, because
        // an active OpenGL context cannot safely traverse a thread boundary
        let context = unsafe {
            let c = windowed_context.make_current().unwrap();
            gl::load_with(|symbol| c.get_proc_address(symbol) as *const _);
            c
        };

        let mut window_aspect_ratio = INITIAL_SCREEN_W as f32 / INITIAL_SCREEN_H as f32;

        // Set up openGL
        unsafe {
            gl::Enable(gl::DEPTH_TEST);
            gl::DepthFunc(gl::LESS);
            gl::Enable(gl::CULL_FACE);
            gl::Disable(gl::MULTISAMPLE);
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            gl::Enable(gl::DEBUG_OUTPUT_SYNCHRONOUS);
            gl::DebugMessageCallback(Some(util::debug_callback), ptr::null());

            // Print some diagnostics
            println!("{}: {}", util::get_gl_string(gl::VENDOR), util::get_gl_string(gl::RENDERER));
            println!("OpenGL\t: {}", util::get_gl_string(gl::VERSION));
            println!("GLSL\t: {}", util::get_gl_string(gl::SHADING_LANGUAGE_VERSION));
        }

        // == // Set up your VAO around here

        //let my_vao = unsafe { 1337 };
        let vertices: Vec<f32> = vec![
            // VIEWING WINDOW
                // 1.0, -1.0, 0.0,  // 0
                // 0.0, 1.0, 0.0,  // 1
                // -1.0, -1.0, 0.0, // 2
            
            /*TASK 3.a */
            // coordinates to draw a circle

            /*TASK 2.d*/
            // Triangle 1
            -0.6, -0.8,-0.8, // 0
            0.0,-0.4,0.0, // 1
            -0.8,-0.2,-0.6, // 2

            // Triangle 2
            1.0, 0.8,-0.8, // 3
            0.0, 1.0, 0.0, // 4
            0.8, 0.5, 0.6, // 5
            

            /*TASK 2.c
                // -- Triangle 1
                0.25, -0.25, 0.0,  // 0
                0.0, 0.25, 0.0,  // 1
                -0.25, -0.25, 0.0, // 2
                // -- Triangle 2
                1.0, 0.8,-0.8, // 3
                0.0, 1.0, 0.0, // 4
                0.8, 0.5, 0.6, // 5
                // -- Triangle 3
                -0.6, -0.8,-0.8, // 6
                0.0,-0.4,0.0, // 7
                -0.8,-0.2,-0.6 // 8
            */
            
            /*TASK 1
                // Triangle 1
                -0.25, 0.56, 0.0, // 0 
                0.25, 0.56, 0.0, // 1
                0.0,  0.83, 0.0, // 2
                // Triangle 2
                -0.15, 0.54, 0.0,  // 3
                0.0, 0.36, 0.0, // 4
                0.15, 0.54, 0.0, // 5
                // Triangle 3
                -0.35, -0.24, 0.0, // 6
                0.35, -0.24, 0.0, // 7
                0.0, 0.34, 0.0, // 8
                // Triangle 4
                -0.17, -0.66, 0.0, // 9
                -0.07, -0.46, 0.0, // 10
                -0.17, -0.26, 0.0, // 11
                // Triangle 5
                0.17, -0.66, 0.0, // 12
                0.17, -0.26, 0.0, // 13
                0.07, -0.46, 0.0, // 14
            */
        ];
        

        let indices: Vec<u32> = vec![
            /*InitialOrder
                2,0,1,
                5,3,4,
                8,6,7*/
                
            /*Task2.d*/
                // Triangle 1
                2,0,1,
                // Triangle 2
                5,3,4

            /*Task2.c
                //Order1
                2,0,1,
                5,3,4,
                8,6,7*/

            /*TASK 1
            0, 1, 2, 
            3, 4, 5, 
            6, 7, 8, 
            9, 10, 11, 
            12, 13, 14   
            */
        ];

        let colors: Vec<f32> =vec![
            1.0, 0.0, 0.0, 0.9,
            0.0, 1.0, 0.0, 0.7,
            0.0, 0.0, 1.0, 0.5,
            1.0, 1.0, 0.0, 0.3,
            0.0, 1.0, 1.0, 0.1,
            1.0, 0.0, 1.0, 0.6,
        ];

        // Draw the triangles
        let vao = unsafe {create_vao(&vertices, &indices, &colors)};

        // == // Set up your shaders here

        // Basic usage of shader helper:
        // The example code below creates a 'shader' object.
        // It which contains the field `.program_id` and the method `.activate()`.
        // The `.` in the path is relative to `Cargo.toml`.
        // This snippet is not enough to do the exercise, and will need to be modified (outside
        // of just using the correct path), but it only needs to be called once

        
        let simple_shader = unsafe {
            shader::ShaderBuilder::new()
                .attach_file("./shaders/simple.vert")
                .attach_file("./shaders/simple.frag")
                .link()
                .activate()
        };     


        // Used to demonstrate keyboard handling for exercise 2.
        let mut _arbitrary_number = 0.0; // feel free to remove


        // The main rendering loop
        let first_frame_time = std::time::Instant::now();
        let mut previous_frame_time = first_frame_time;
        loop {
            // Compute time passed since the previous frame and since the start of the program
            let now = std::time::Instant::now();
            let elapsed = now.duration_since(first_frame_time).as_secs_f32();
            let delta_time = now.duration_since(previous_frame_time).as_secs_f32();
            previous_frame_time = now;

            // Handle resize events
            if let Ok(mut new_size) = window_size.lock() {
                if new_size.2 {
                    context.resize(glutin::dpi::PhysicalSize::new(new_size.0, new_size.1));
                    window_aspect_ratio = new_size.0 as f32 / new_size.1 as f32;
                    (*new_size).2 = false;
                    println!("Window was resized to {}x{}", new_size.0, new_size.1);
                    unsafe { gl::Viewport(0, 0, new_size.0 as i32, new_size.1 as i32); }
                }
            }

            // Handle keyboard input
            if let Ok(keys) = pressed_keys.lock() {
                for key in keys.iter() {
                    match key {
                        // The `VirtualKeyCode` enum is defined here:
                        //    https://docs.rs/winit/0.25.0/winit/event/enum.VirtualKeyCode.html

                        VirtualKeyCode::A => {
                            _arbitrary_number += delta_time;
                        }
                        VirtualKeyCode::D => {
                            _arbitrary_number -= delta_time;
                        }


                        // default handler:
                        _ => { }
                    }
                }
            }
            // Handle mouse movement. delta contains the x and y movement of the mouse since last frame in pixels
            if let Ok(mut delta) = mouse_delta.lock() {

                // == // Optionally access the accumulated mouse movement between
                // == // frames here with `delta.0` and `delta.1`

                *delta = (0.0, 0.0); // reset when done
            }

            // == // Please compute camera transforms here (exercise 2 & 3)


            unsafe {
                // Clear the color and depth buffers
                gl::ClearColor(0.035, 0.046, 0.078, 1.0); // night sky, full opacity
                gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);


                // == // Issue the necessary gl:: commands to draw your scene here
                gl::DrawElements(gl::TRIANGLES, 15,gl::UNSIGNED_INT,ptr::null());


            }

            // Display the new color buffer on the display
            context.swap_buffers().unwrap(); // we use "double buffering" to avoid artifacts
        }
    });


    // == //
    // == // From here on down there are only internals.
    // == //


    // Keep track of the health of the rendering thread
    let render_thread_healthy = Arc::new(RwLock::new(true));
    let render_thread_watchdog = Arc::clone(&render_thread_healthy);
    thread::spawn(move || {
        if !render_thread.join().is_ok() {
            if let Ok(mut health) = render_thread_watchdog.write() {
                println!("Render thread panicked!");
                *health = false;
            }
        }
    });

    // Start the event loop -- This is where window events are initially handled
    el.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        // Terminate program if render thread panics
        if let Ok(health) = render_thread_healthy.read() {
            if *health == false {
                *control_flow = ControlFlow::Exit;
            }
        }

        match event {
            Event::WindowEvent { event: WindowEvent::Resized(physical_size), .. } => {
                println!("New window size received: {}x{}", physical_size.width, physical_size.height);
                if let Ok(mut new_size) = arc_window_size.lock() {
                    *new_size = (physical_size.width, physical_size.height, true);
                }
            }
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                *control_flow = ControlFlow::Exit;
            }
            // Keep track of currently pressed keys to send to the rendering thread
            Event::WindowEvent { event: WindowEvent::KeyboardInput {
                    input: KeyboardInput { state: key_state, virtual_keycode: Some(keycode), .. }, .. }, .. } => {

                if let Ok(mut keys) = arc_pressed_keys.lock() {
                    match key_state {
                        Released => {
                            if keys.contains(&keycode) {
                                let i = keys.iter().position(|&k| k == keycode).unwrap();
                                keys.remove(i);
                            }
                        },
                        Pressed => {
                            if !keys.contains(&keycode) {
                                keys.push(keycode);
                            }
                        }
                    }
                }

                // Handle Escape and Q keys separately
                match keycode {
                    Escape => { *control_flow = ControlFlow::Exit; }
                    Q      => { *control_flow = ControlFlow::Exit; }
                    _      => { }
                }
            }
            Event::DeviceEvent { event: DeviceEvent::MouseMotion { delta }, .. } => {
                // Accumulate mouse movement
                if let Ok(mut position) = arc_mouse_delta.lock() {
                    *position = (position.0 + delta.0 as f32, position.1 + delta.1 as f32);
                }
            }
            _ => { }
        }
    });
}
