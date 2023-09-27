// Uncomment these following global attributes to silence most warnings of "low" interest:
#![allow(dead_code)]
#![allow(non_snake_case)]
#![allow(unreachable_code)]
#![allow(unused_mut)]
#![allow(unused_unsafe)]
#![allow(unused_variables)]

extern crate nalgebra_glm as glm;

use std::{ mem, ptr, os::raw::c_void, thread};
use std::sync::{Mutex, Arc, RwLock};

mod shader;
mod util;
mod camera;

use glutin::event::{Event, WindowEvent, DeviceEvent, KeyboardInput, ElementState::{Pressed, Released}, VirtualKeyCode::{self,*}};
use glutin::event_loop::ControlFlow;
use glutin::window::CursorGrabMode;

// initial window size
const INITIAL_SCREEN_W: u32 = 800;
const INITIAL_SCREEN_H: u32 = 600;

// == // Helper functions to make interacting with OpenGL a little bit prettier. You WILL need these! // == //

// Get the size of an arbitrary array of numbers measured in bytes
// Example usage:  byte_size_of_array(my_array)
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


// == // Generate your VAO here
unsafe fn create_vao(vertices: &Vec<f32>, indices: &Vec<u32>, colors: &Vec<f32>) -> u32 {

    let mut vao_id: u32 = 0;
    gl::GenVertexArrays(1,&mut vao_id);
    gl::BindVertexArray(vao_id);

    //info en Opengl pdf pg 12 size: bloq.1, data: bloq.6
    let mut vbo_id: u32 = 0;
    gl::GenBuffers(1,&mut vbo_id);
    gl::BindBuffer(gl::ARRAY_BUFFER, vbo_id);
    gl::BufferData(
        gl::ARRAY_BUFFER,
        byte_size_of_array(vertices),
        pointer_to_array(vertices),
        gl::STATIC_DRAW);

    let vertex_attribute_index: u32 = 0;
    gl::VertexAttribPointer(
        vertex_attribute_index,
        3,
        gl::FLOAT,
        gl::FALSE,
        size_of::<f32>() * 3,
        offset::<c_void>(0));
    gl::EnableVertexAttribArray(vertex_attribute_index);

    let mut cbo_id: u32 = 1;
    gl::GenBuffers(1, &mut cbo_id);
    gl::BindBuffer(gl::ARRAY_BUFFER, cbo_id);
    gl::BufferData(
    gl::ARRAY_BUFFER,                 // Target
    byte_size_of_array(colors), // Size
    pointer_to_array(colors),   // Data
    gl::STATIC_DRAW,                  // Usage
    );

    let color_attrbute_index: u32 = 2;
    gl::VertexAttribPointer(
    color_attrbute_index,                        // Index
    4,                          // Size
    gl::FLOAT,                                   // Type
    gl::FALSE,                                   // Normalized
    size_of::<f32>() * 4, // Stride
    offset::<c_void>(0));                   // Offset
    gl::EnableVertexAttribArray(color_attrbute_index);
    
    let mut ibo_id: u32 = 0;
    gl::GenBuffers(1,&mut ibo_id);
    gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ibo_id);
    gl::BufferData(
        gl::ELEMENT_ARRAY_BUFFER,
        byte_size_of_array(indices),
        pointer_to_array(indices),
        gl::STATIC_DRAW);

    return vao_id;
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
    windowed_context
        .window()
        .set_cursor_grab(CursorGrabMode::Confined)
        .expect("failed to grab cursor");
    windowed_context.window().set_cursor_visible(false);

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

        let vertex :i32 = 0;
        let shader: shader::Shader;

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

            shader = shader::ShaderBuilder::new()
                .attach_file("shaders/simple.vert")
                .attach_file("shaders/simple.frag")
                .link();
            shader.activate();
        }


        //anadido por mi PATATA
        let vertices: Vec<f32> = vec![
            -0.25, 0.56, 0.0, // Point 0
            0.25, 0.56, 0.0, // Point 1
            0.0,  0.83, 0.0, // Point 2
            -0.15, 0.54, 0.0,
            0.0, 0.36, 0.0,
            0.15, 0.54, 0.0,
            -0.35, -0.24, 0.0,
            0.35, -0.24, 0.0,
            0.0, 0.34, 0.0,
            -0.17, -0.66, 0.0,
            -0.07, -0.46, 0.0,
            -0.17, -0.26, 0.0,
            0.17, -0.66, 0.0,
            0.17, -0.26, 0.0,
            0.07, -0.46, 0.0

            //task 2a
            /*0.0, 0.5, 0.0,
            -0.5, -0.5, 0.0,
            0.5, -0.5, 0.0,
            -0.5, 0.5, -0.3,
            -0.75, -0.8, -0.3,
            0.85, -0.9, -0.3,
            0.92, 0.92, -0.5,
            -0.1, 0.0, -0.5,
            0.4, -0.75, -0.5,

            //task 2b 2
            0.0, 0.5, -0.8,
            -0.5, -0.5, -0.8,
            0.5, -0.5, -0.8,
            -0.5, 0.5, -0.3,
            -0.75, -0.8, -0.3,
            0.85, -0.9, -0.3,
            0.92, 0.92, 0.5,
            -0.1, 0.0, 0.5,
            0.4, -0.75, 0.5,*/
        ];
        let indices: Vec<u32> = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14];
        let colors: Vec<f32> = vec![
            0.0, 0.0, 1.0, 0.4, //
            0.0, 0.8, 1.0, 0.4, //
            0.8, 0.0, 1.0, 0.4, //

            1.0, 1.0, 0.0, 0.4, //
            1.0, 0.0, 0.0, 0.4, //
            1.0, 1.0, 1.0, 0.4, //

            0.0, 1.0, 1.0, 0.4, //
            1.0, 1.0, 0.0, 0.4, //
            0.0, 1.0, 1.0, 0.4, //

            1.0, 1.0, 1.0, 0.4, //
            1.0, 0.0, 0.0, 0.4, //
            1.0, 0.0, 1.0, 0.4, //

            0.0, 0.0, 1.0, 0.4, //
            0.0, 0.8, 1.0, 0.4, //
            0.8, 0.0, 1.0, 0.4, 

            //task 2a y 2b 2
            /*1.0, 0.0, 0.0, 0.5,
            1.0, 0.0, 0.0, 0.5,
            1.0, 0.0, 0.0, 0.5,
            // verde
            0.0, 1.0, 0.0, 0.5,
            0.0, 1.0, 0.0, 0.5,
            0.0, 1.0, 0.0, 0.5,
            // azul
            0.0, 0.0, 1.0, 0.5,
            0.0, 0.0, 1.0, 0.5,
            0.0, 0.0, 1.0, 0.5,*/

            //task 2b 1
            //azul
            /*0.0, 0.0, 1.0, 0.5,
            0.0, 0.0, 1.0, 0.5,
            0.0, 0.0, 1.0, 0.5,
            //verde
            0.0, 1.0, 0.0, 0.5,
            0.0, 1.0, 0.0, 0.5,
            0.0, 1.0, 0.0, 0.5,
            //rojo
            1.0, 0.0, 0.0, 0.5,
            1.0, 0.0, 0.0, 0.5,
            1.0, 0.0, 0.0, 0.5,*/
        ];

        let my_vao = unsafe { create_vao(&vertices, &indices, &colors) };

        // Used to demonstrate keyboard handling for exercise 2.
        let mut _arbitrary_number = 0.0; // feel free to remove


        // The main rendering loop
        let first_frame_time = std::time::Instant::now();
        let mut previous_frame_time = first_frame_time;

        let mut camera = camera::Camera::new();

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

            // Handle keyboard input - camera movement
            if let Ok(keys) = pressed_keys.lock() {
                let distance = 9.1 * delta_time;
                let rotation = 3.1 * delta_time;
                for key in keys.iter() {
                    match key {
                        // The `VirtualKeyCode` enum is defined here:
                        //    https://docs.rs/winit/0.25.0/winit/event/enum.VirtualKeyCode.html

                        VirtualKeyCode::Space => camera.forward(distance),
                        VirtualKeyCode::Left => camera.left(distance),
                        VirtualKeyCode::Delete => camera.backward(distance),
                        VirtualKeyCode::Right => camera.right(distance),
                        VirtualKeyCode::Up => camera.up(distance),
                        VirtualKeyCode::Down => camera.down(distance),
                        VirtualKeyCode::R => camera.reset(),

                        VirtualKeyCode::Up => camera.update_angle(rotation),
                        VirtualKeyCode::Down => camera.update_angle(-rotation),
                        VirtualKeyCode::Left => camera.update_yaw(-rotation),
                        VirtualKeyCode::Right => camera.update_yaw(rotation),

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

            // PATATA
            let composite_matrix: glm::Mat4 = glm::identity();


            unsafe {
                gl::Uniform1f(
                    shader.get_uniform_location("elapsed"), elapsed,
                );
                gl::UniformMatrix4fv(
                    shader.get_uniform_location( "composite_matrix"),
                    1,
                    0,
                composite_matrix.as_ptr(),
                );
            }

            let ratio = INITIAL_SCREEN_H as f32 / INITIAL_SCREEN_W as f32;
            let perspective: glm::Mat4 = camera.projection(ratio);
            let perspective_translation = camera.translate(glm::vec3(0.0, 0.0, -2.0));

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

    //************* window::run START *************/
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
    //************* window::run END *************/
}
