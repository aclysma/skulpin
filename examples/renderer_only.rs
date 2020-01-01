// This example shows how to use the renderer directly. This allows full control of winit
// and the update loop

use skulpin::CoordinateSystemHelper;

fn main() {
    // Setup logging
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Debug)
        .init();

    // Create the winit event loop
    let event_loop = winit::event_loop::EventLoop::<()>::with_user_event();

    // Set up the coordinate system to be fixed at 900x600, and use this as the default window size
    // This means the drawing code can be written as though the window is always 900x600. The
    // output will be automatically scaled so that it's always visible.
    let logical_size = winit::dpi::LogicalSize::new(900.0, 600.0);
    let visible_range = skulpin::skia_safe::Rect {
        left: 0.0,
        right: logical_size.width as f32,
        top: 0.0,
        bottom: logical_size.height as f32,
    };
    let scale_to_fit = skulpin::skia_safe::matrix::ScaleToFit::Center;

    // Create a single window
    let window = winit::window::WindowBuilder::new()
        .with_title("Skulpin")
        .with_inner_size(logical_size)
        .build(&event_loop)
        .expect("Failed to create window");

    #[cfg(feature = "with_imgui")]
    let mut imgui_manager = init_imgui_manager(&window);
    #[cfg(feature = "with_imgui")]
    imgui_manager.begin_frame(&window);

    // Create the renderer, which will draw to the window
    let renderer = skulpin::RendererBuilder::new()
        .use_vulkan_debug_layer(true)
        .coordinate_system(skulpin::CoordinateSystem::VisibleRange(
            visible_range,
            scale_to_fit,
        ))
        .build(
            &window,
            #[cfg(feature = "with_imgui")]
            &mut imgui_manager,
        );

    // Check if there were error setting up vulkan
    if let Err(e) = renderer {
        println!("Error during renderer construction: {:?}", e);
        return;
    }

    let mut renderer = renderer.unwrap();

    // Increment a frame count so we can render something that moves
    let mut frame_count = 0;

    // Start the window event loop. Winit will not return once run is called. We will get notified
    // when important events happen.
    event_loop.run(move |event, _window_target, control_flow| {
        #[cfg(feature = "with_imgui")]
        imgui_manager.handle_event(&window, &event);

        match event {
            //
            // Halt if the user requests to close the window
            //
            winit::event::Event::WindowEvent {
                event: winit::event::WindowEvent::CloseRequested,
                ..
            } => *control_flow = winit::event_loop::ControlFlow::Exit,

            //
            // Close if the escape key is hit
            //
            winit::event::Event::WindowEvent {
                event:
                    winit::event::WindowEvent::KeyboardInput {
                        input:
                            winit::event::KeyboardInput {
                                virtual_keycode: Some(winit::event::VirtualKeyCode::Escape),
                                ..
                            },
                        ..
                    },
                ..
            } => *control_flow = winit::event_loop::ControlFlow::Exit,

            //
            // Request a redraw any time we finish processing events
            //
            winit::event::Event::MainEventsCleared => {
                // Queue a RedrawRequested event.
                window.request_redraw();
            }

            //
            // Redraw
            //
            winit::event::Event::RedrawRequested(_window_id) => {
                if let Err(e) = renderer.draw(
                    &window,
                    #[cfg(feature = "with_imgui")]
                    &mut imgui_manager,
                    |canvas,
                     coordinate_system_helper,
                     #[cfg(feature = "with_imgui")] imgui_manager| {
                        draw(canvas, coordinate_system_helper, frame_count);
                        frame_count += 1;

                        #[cfg(feature = "with_imgui")]
                        {
                            imgui_manager.with_ui(|ui: &mut imgui::Ui| {
                                let mut show_demo = true;
                                ui.show_demo_window(&mut show_demo);

                                ui.main_menu_bar(|| {
                                    ui.menu(imgui::im_str!("File"), true, || {
                                        if imgui::MenuItem::new(imgui::im_str!("New")).build(ui) {
                                            log::info!("clicked");
                                        }
                                    });
                                });
                            });
                        }
                    },
                ) {
                    println!("Error during draw: {:?}", e);
                    *control_flow = winit::event_loop::ControlFlow::Exit
                }
            }

            //
            // Ignore all other events
            //
            _ => {}
        }
    });
}

/// Called when winit passes us a WindowEvent::RedrawRequested
fn draw(
    canvas: &mut skia_safe::Canvas,
    _coordinate_system_helper: &CoordinateSystemHelper,
    frame_count: i32,
) {
    // Generally would want to clear data every time we draw
    canvas.clear(skia_safe::Color::from_argb(0, 0, 0, 255));

    // Floating point value constantly moving between 0..1 to generate some movement
    let f = ((frame_count as f32 / 30.0).sin() + 1.0) / 2.0;

    // Make a color to draw with
    let mut paint = skia_safe::Paint::new(skia_safe::Color4f::new(1.0 - f, 0.0, f, 1.0), None);
    paint.set_anti_alias(true);
    paint.set_style(skia_safe::paint::Style::Stroke);
    paint.set_stroke_width(2.0);

    // Draw a line
    canvas.draw_line(
        skia_safe::Point::new(100.0, 500.0),
        skia_safe::Point::new(800.0, 500.0),
        &paint,
    );

    // Draw a circle
    canvas.draw_circle(
        skia_safe::Point::new(200.0 + (f * 500.0), 420.0),
        50.0,
        &paint,
    );

    // Draw a rectangle
    canvas.draw_rect(
        skia_safe::Rect {
            left: 10.0,
            top: 10.0,
            right: 890.0,
            bottom: 590.0,
        },
        &paint,
    );

    //TODO: draw_bitmap

    let mut font = skia_safe::Font::default();
    font.set_size(100.0);

    canvas.draw_str("Hello Skulpin", (65, 200), &font, &paint);
    canvas.draw_str("Hello Skulpin", (68, 203), &font, &paint);
    canvas.draw_str("Hello Skulpin", (71, 206), &font, &paint);
}

#[cfg(feature = "with_imgui")]
fn init_imgui(window: &winit::window::Window) -> imgui::Context {
    use imgui::Context;

    let mut imgui = Context::create();
    {
        // Fix incorrect colors with sRGB framebuffer
        fn imgui_gamma_to_linear(col: [f32; 4]) -> [f32; 4] {
            let x = col[0].powf(2.2);
            let y = col[1].powf(2.2);
            let z = col[2].powf(2.2);
            let w = 1.0 - (1.0 - col[3]).powf(2.2);
            [x, y, z, w]
        }

        let style = imgui.style_mut();
        for col in 0..style.colors.len() {
            style.colors[col] = imgui_gamma_to_linear(style.colors[col]);
        }
    }

    imgui.set_ini_filename(None);

    // In the examples we only use integer DPI factors, because the UI can get very blurry
    // otherwise. This might or might not be what you want in a real application.
    let hidpi_factor = window.hidpi_factor().round();
    let font_size = (16.0 * hidpi_factor) as f32;
    imgui.fonts().add_font(&[imgui::FontSource::TtfData {
        data: include_bytes!("../fonts/mplus-1p-regular.ttf"),
        size_pixels: font_size,
        config: None,
    }]);

    imgui.io_mut().font_global_scale = (1.0 / hidpi_factor) as f32;

    return imgui;
}

#[cfg(feature = "with_imgui")]
pub fn init_imgui_manager(window: &winit::window::Window) -> skulpin::ImguiManager {
    let mut imgui_context = init_imgui(&window);
    let mut imgui_platform = imgui_winit_support::WinitPlatform::init(&mut imgui_context);

    imgui_platform.attach_window(
        imgui_context.io_mut(),
        &window,
        imgui_winit_support::HiDpiMode::Rounded,
    );

    skulpin::ImguiManager::new(imgui_context, imgui_platform)
}
