use std::{convert::TryFrom, sync::Arc};

use log::{info};
use bytemuck::{Pod, Zeroable};
use egui::{epaint::Shadow, style::Margin, vec2, Align, Align2, Color32, Frame, Rounding, Window};
use egui_winit_vulkano::{Gui, GuiConfig};
use vulkano::{
    buffer::{Buffer, BufferUsage, BufferCreateInfo, Subbuffer},
    command_buffer::{
        allocator::StandardCommandBufferAllocator, AutoCommandBufferBuilder,
        CommandBufferInheritanceInfo, CommandBufferUsage, RenderPassBeginInfo, SubpassContents,
    },
    device::{Device, Queue},
    format::Format,
    image::{ImageAccess, SampleCount},
    memory::allocator::{AllocationCreateInfo, MemoryUsage, StandardMemoryAllocator},
    pipeline::{
        graphics::{
            input_assembly::InputAssemblyState,
            multisample::MultisampleState,
            vertex_input::BuffersDefinition,
            viewport::{Viewport, ViewportState},
        },
        GraphicsPipeline, ComputePipeline,
    },
    render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass, Subpass},
    sync::GpuFuture,
};
use vulkano_util::{
    context::{VulkanoConfig, VulkanoContext},
    renderer::SwapchainImageView,
    window::{VulkanoWindows, WindowDescriptor},
};
use winit::{
    event::{Event, WindowEvent, ElementState},
    event_loop::{ControlFlow, EventLoop},
};

// Render a triangle (scene) and a gui from a subpass on top of it (with some transparent fill)

pub fn main() {
    // Init logger
    //egui_logger::init().unwrap();

    // Winit event loop
    let event_loop = EventLoop::new();
    // Vulkano context
    let context = VulkanoContext::new(VulkanoConfig::default());
    // Vulkano windows (create one)
    let mut windows = VulkanoWindows::default();
    windows.create_window(&event_loop, &context, &WindowDescriptor::default(), |ci| {
        ci.image_format = Some(vulkano::format::Format::B8G8R8A8_SRGB)
    });
    // Create out gui pipeline
    let mut gui_pipeline = SimpleGuiPipeline::new(
        context.graphics_queue().clone(),
        windows.get_primary_renderer_mut().unwrap().swapchain_format(),
        context.memory_allocator(),
    );
    // Create simple gui
    let mut gui = Gui::new(
        &event_loop,
        windows.get_primary_renderer_mut().unwrap().surface(),
        windows.get_primary_renderer_mut().unwrap().graphics_queue(),
        GuiConfig {
            preferred_format: Some(vulkano::format::Format::B8G8R8A8_SRGB),
            is_overlay: true,
            ..Default::default()
        },
    );

    // Create gui state (pass anything your state requires)
    event_loop.run(move |event, _, control_flow| {
        let renderer = windows.get_primary_renderer_mut().unwrap();
        match event {
            Event::WindowEvent { event, window_id } if window_id == renderer.window().id() => {
                // Update Egui integration so the UI works!
                let _pass_events_to_game = !gui.update(&event);
                match event {
                    WindowEvent::Resized(size) => {
                        // Resize with 0 width and height is used by winit to signal a minimize event on Windows.
                        // See: https://github.com/rust-windowing/winit/issues/208
                        // This solves an issue where the app would panic when minimizing on Windows.
                        let w = size.width;
                        let h = size.height;
                        info!("Resized {w}x{h}");
                        if size.width > 0 && size.height > 0 {
                            gui_pipeline.minimized = false;
                            renderer.resize();
                        } else {
                            gui_pipeline.minimized = true;
                        }
                    }
                    WindowEvent::ScaleFactorChanged { .. } => {
                        info!("ScaleFactorChanged");
                        renderer.resize();
                    }
                    WindowEvent::CloseRequested => {
                        *control_flow = ControlFlow::Exit;
                    }
                    WindowEvent::Focused(focused) => {
                        if focused {
                            info!("*** Have focus");
                        } else {
                            info!("*** Lost focus");
                        }
                    }
                    WindowEvent::KeyboardInput { device_id: _, input, is_synthetic: _ } => {
                        let keycode = input.scancode;
                        let state = input.state == ElementState::Pressed;
                        info!("{keycode}: {state}");
                    }
                    _ => (),
                }
            }
            Event::Suspended => {
                info!("*** Suspended");
            }
            Event::Resumed => {
                info!("*** Resumed");
            }
            Event::RedrawRequested(window_id) if !gui_pipeline.minimized && window_id == window_id => {
                // Set immediate UI in redraw here
                gui.immediate_ui(|gui| {
                    let ctx = gui.context();
                    /*Window::new("Transparent Window")
                        .anchor(Align2([Align::RIGHT, Align::TOP]), vec2(-545.0, 500.0))
                        .resizable(false)
                        .default_width(300.0)
                        .frame(
                            Frame::none()
                                .fill(Color32::from_white_alpha(125))
                                .shadow(Shadow {
                                    extrusion: 8.0,
                                    color: Color32::from_black_alpha(125),
                                })
                                .rounding(Rounding::same(5.0))
                                .inner_margin(Margin::same(10.0)),
                        )
                        .show(&ctx, |ui| {
                            ui.colored_label(Color32::BLACK, "Content :)");
                        });*/
                    Window::new("Window")
                        .resizable(false)
                        .show(&ctx, |ui| {
                            ui.label("Hello world!");
                            ui.label("See https://github.com/emilk/egui for how to make other UI elements");
                        });
                    /*Window::new("Log")
                        .show(&ctx, |ui| {
                            // draws the logger ui.
                            egui_logger::logger_ui(ui);
                        });*/
                });

                // Render

                // Acquire swapchain future
                let before_future = renderer.acquire().unwrap();

                // Render scene
                // rendering done in gui_pipeline.render...

                // Render gui
                let after_future =
                    gui_pipeline.render(before_future, renderer.swapchain_image_view(), &mut gui);

                // Present swapchain
                renderer.present(after_future, true);
            }
            Event::MainEventsCleared => {
                // Send request_redraw to render a new frame regardless
                renderer.window().request_redraw();
            }
            _ => (),
        }
    });
}

struct SimpleGuiPipeline {
    queue: Arc<Queue>,
    render_pass: Arc<RenderPass>,
    pipeline: Arc<GraphicsPipeline>,
    vertex_buffer: Subbuffer<[Vertex]>,
    command_buffer_allocator: StandardCommandBufferAllocator,
    minimized: bool,
}

impl SimpleGuiPipeline {
    pub fn new(
        queue: Arc<Queue>,
        image_format: vulkano::format::Format,
        allocator: &StandardMemoryAllocator,
    ) -> Self {
        let render_pass = Self::create_render_pass(queue.device().clone(), image_format);
        let pipeline = Self::create_pipeline(queue.device().clone(), render_pass.clone());

        let vertex_buffer = {
            Buffer::from_iter(
                allocator,
                BufferCreateInfo { usage: BufferUsage::VERTEX_BUFFER, ..Default::default() },
                AllocationCreateInfo{ usage: MemoryUsage::Upload, ..Default::default() },
                [
                    Vertex { position: [-0.5, -0.25], color: [1.0, 0.0, 0.0, 1.0] },
                    Vertex { position: [0.0, 0.5], color: [0.0, 1.0, 0.0, 1.0] },
                    Vertex { position: [0.25, -0.1], color: [0.0, 0.0, 1.0, 1.0] },
                ]
                .iter()
                .cloned(),
            )
            .expect("failed to create buffer")
        };

        // Create an allocator for command-buffer data
        let command_buffer_allocator =
            StandardCommandBufferAllocator::new(queue.device().clone(), Default::default());

        Self { queue, render_pass, pipeline/*, subpass*/, vertex_buffer, command_buffer_allocator, minimized: false }
    }

    fn create_render_pass(device: Arc<Device>, format: Format) -> Arc<RenderPass> {
        vulkano::ordered_passes_renderpass!(
            device,
            attachments: {
                color: {
                    load: Clear,
                    store: Store,
                    format: format,
                    samples: SampleCount::Sample1,
                }
            },
            passes: [
                { color: [color], depth_stencil: {}, input: [] } // Single pass
            ]
        )
        .unwrap()
    }

    fn create_pipeline(
        device: Arc<Device>,
        render_pass: Arc<RenderPass>,
    ) -> Arc<GraphicsPipeline> {//(Arc<GraphicsPipeline>, Subpass) {
        let vs = vs::load(device.clone()).expect("failed to create shader module");
        let fs = fs::load(device.clone()).expect("failed to create shader module");

        GraphicsPipeline::start()
            .vertex_input_state(BuffersDefinition::new().vertex::<Vertex>())
            .vertex_shader(vs.entry_point("main").unwrap(), ())
            .input_assembly_state(InputAssemblyState::new())
            .fragment_shader(fs.entry_point("main").unwrap(), ())
            .viewport_state(ViewportState::viewport_dynamic_scissor_irrelevant())
            .render_pass(Subpass::from(render_pass, 0).unwrap())
            .multisample_state(MultisampleState {
                rasterization_samples: SampleCount::Sample1,
                ..Default::default()
            })
            .build(device)
            .unwrap()
    }

    fn create_compute_pipeline(device: Arc<Device>) -> Arc<ComputePipeline> {
        let pipeline = {
            mod cs {
                vulkano_shaders::shader! {
                    ty: "compute",
                    src: r"
                        #version 450
                        layout(constant_id = 0) const int multiple = 12;
                        layout(local_size_x = GROUP_X, local_size_y = 1, local_size_z = 1) in;
                        layout(set = 0, binding = 0) buffer Data {
                            uint data[];
                        };
                        struct SDFShape {
                            vec4 halfSizeAndSoftRadius;
                        };
                        layout(set = 0, binding = 1) readonly buffer Shapes {
                            SDFShape shapes[];
                        };
                        void main() {
                            uint idx = gl_GlobalInvocationID.x;
                            data[idx] *= multiple;
                        }
                    ",
                    define: [
                        ("GROUP_X", "64")
                    ],
                    //dump: true,
                }
            }

            let shader = cs::load(device.clone()).unwrap();
            let spec_consts = cs::SpecializationConstants {
                multiple: 12,
            };
            ComputePipeline::new(
                device.clone(),
                shader.entry_point("main").unwrap(),
                &spec_consts,
                None,
                |_| {},
            )
            .unwrap()
        };
        pipeline        
    }

    pub fn render(
        &mut self,
        before_future: Box<dyn GpuFuture>,
        image: SwapchainImageView,
        gui: &mut Gui,
    ) -> Box<dyn GpuFuture> {
        let mut builder = AutoCommandBufferBuilder::primary(
            &self.command_buffer_allocator,
            self.queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();

        let dimensions = image.image().dimensions().width_height();
        let framebuffer = Framebuffer::new(self.render_pass.clone(), FramebufferCreateInfo {
            attachments: vec![image.clone()],
            ..Default::default()
        })
        .unwrap();

        // Begin render pipeline commands
        builder
            .begin_render_pass(
                RenderPassBeginInfo {
                    clear_values: vec![Some([0.0, 0.0, 0.0, 1.0].into())],
                    ..RenderPassBeginInfo::framebuffer(framebuffer)
                },
                SubpassContents::Inline,
            )
            .unwrap()
            .bind_pipeline_graphics(self.pipeline.clone())
            .set_viewport(0, vec![Viewport {
                origin: [0.0, 0.0],
                dimensions: [dimensions[0] as f32, dimensions[1] as f32],
                depth_range: 0.0..1.0,
            }])
            .bind_vertex_buffers(0, self.vertex_buffer.clone())
            .draw(self.vertex_buffer.len() as u32, 1, 0, 0)
            .unwrap()
            .end_render_pass()
            .unwrap();

        let command_buffer = builder.build().unwrap();
        let after_future = before_future.then_execute(self.queue.clone(), command_buffer).unwrap();

        // Render gui as overlay
        let after_gui_future = gui.draw_on_image(after_future, image);
        after_gui_future.boxed()
/*
        let mut builder = AutoCommandBufferBuilder::primary(
            &self.command_buffer_allocator,
            self.queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();

        let dimensions = image.image().dimensions().width_height();
        let framebuffer = Framebuffer::new(self.render_pass.clone(), FramebufferCreateInfo {
            attachments: vec![image],
            ..Default::default()
        })
        .unwrap();

        // Begin render pipeline commands
        builder
            .begin_render_pass(
                RenderPassBeginInfo {
                    clear_values: vec![Some([0.0, 0.0, 0.0, 1.0].into())],
                    ..RenderPassBeginInfo::framebuffer(framebuffer)
                },
                SubpassContents::SecondaryCommandBuffers,
            )
            .unwrap();

        // Render first draw pass
        let mut secondary_builder = AutoCommandBufferBuilder::secondary(
            &self.command_buffer_allocator,
            self.queue.queue_family_index(),
            CommandBufferUsage::MultipleSubmit,
            CommandBufferInheritanceInfo {
                render_pass: Some(self.subpass.clone().into()),
                ..Default::default()
            },
        )
        .unwrap();
        secondary_builder
            .bind_pipeline_graphics(self.pipeline.clone())
            .set_viewport(0, vec![Viewport {
                origin: [0.0, 0.0],
                dimensions: [dimensions[0] as f32, dimensions[1] as f32],
                depth_range: 0.0..1.0,
            }])
            .bind_vertex_buffers(0, self.vertex_buffer.clone())
            .draw(self.vertex_buffer.len() as u32, 1, 0, 0)
            .unwrap();
        let cb = secondary_builder.build().unwrap();
        builder.execute_commands(cb).unwrap();

        // Move on to next subpass for gui
        builder.next_subpass(SubpassContents::SecondaryCommandBuffers).unwrap();
        // Draw gui on subpass
        let cb = gui.draw_on_subpass_image(dimensions);
        builder.execute_commands(cb).unwrap();
*/
    }
}

#[repr(C)]
#[derive(Default, Debug, Copy, Clone, Zeroable, Pod)]
struct Vertex {
    position: [f32; 2],
    color: [f32; 4],
}
vulkano::impl_vertex!(Vertex, position, color);

mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        src: "
#version 450
layout(location = 0) in vec2 position;
layout(location = 1) in vec4 color;
layout(location = 0) out vec4 v_color;
void main() {
    gl_Position = vec4(position, 0.0, 1.0);
    v_color = color;
}"
    }
}

mod fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        src: "
#version 450
layout(location = 0) in vec4 v_color;
layout(location = 0) out vec4 f_color;
void main() {
    f_color = v_color;
}"
    }
}