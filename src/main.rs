
use nvg::{Align, Color, Context};
use std::time::Instant;

const SQUARE_SIZE: u32 = 50;

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cell {
    Dead = 0,
    Alive = 1,
}

pub struct Universe {
    width: u32,
    height: u32,
    cells: Vec<Cell>,
    dirty: bool,
}

impl Universe {

    fn new(width: u32, height: u32) -> Universe {
        // let mut rng = rand::thread_rng();
        return Universe {
            width,
            height,
            cells: (0..(width*height)).map(|i| {
                // if i % 2 == 0 || i % 7 == 0 {
                if rand::random::<u8>()%2 == 1 {
                    Cell::Alive
                } else {
                    Cell::Dead
                }
            }).collect(),
            dirty: true,
        };
    }

    fn get_cell_state(&self, row: u32, column: u32) -> Cell {
        let idx = self.get_index(row, column);
        return self.cells[idx];
    }

    fn live_neighbors(&self, row: u32, column: u32) -> u8 {
        let mut count = 0;
        for drow in ([-1, 0, 1] as [i32; 3]).iter().cloned() {
            for dcol in ([-1, 0, 1] as [i32; 3]).iter().cloned() {
                if (drow == 0 && dcol == 0) ||
                    (drow == -1 && row == 0) ||
                    (drow == 1 && row == self.height - 1) ||
                    (dcol == -1 && column == 0) ||
                    (dcol == 1  && column == self.width - 1) {
                    continue;
                }
                let idx = self.get_index(((row as i32) + drow) as u32, ((column as i32) + dcol) as u32);
                count += self.cells[idx] as u8;
            }
        }
        return count;
    }    

    fn get_index(&self, row: u32, column: u32) -> usize {
        (row * self.width + column) as usize
    }
        

    fn tick(&mut self) {
        self.dirty = false;
        let mut next = self.cells.clone();

        for row in 0..self.height {
            for col in 0..self.width {
                let idx = self.get_index(row, col);            
                let cell = self.cells[idx];
                let live_neighbors = self.live_neighbors(row, col);

                let next_cell_state = match (cell, live_neighbors) {
                    (Cell::Alive, x) if x < 2 => Cell::Dead,
                    (Cell::Alive, 2) | (Cell::Alive, 3) => Cell::Alive,
                    (Cell::Alive, x) if x > 3 => Cell::Dead,
                    (Cell::Dead, 3) => Cell::Alive,
                    (otherwise_set_same_state, _) => otherwise_set_same_state,
                };

                if next[idx] != next_cell_state {
                    self.dirty = true;
                }
                next[idx] = next_cell_state;
            }
        }

        self.cells = next;
    }

    fn debug_print(&self) {
        for row in 0..self.height {
            for col in 0..self.width {
                let cell_state = self.get_cell_state(row, col);
    
                if cell_state == Cell::Alive { print!("*"); } else { print!(" "); }
            }
            print!("\n");
        }
        println!("-----------------------------------------------------\n");        
    }
}



fn init(ctx: &mut Context<nvg_gl::Renderer>) -> anyhow::Result<()> {
    ctx.create_font_from_file("roboto", "fonts/Roboto-Bold.ttf").unwrap();
    Ok(())
}

fn update(universe: &Universe, width: f32, height: f32, ctx: &mut Context<nvg_gl::Renderer>) -> anyhow::Result<()> {

    let white_color: Color = Color::rgba(1.0, 1.0, 1.0, 1.0);
    let orange_color: Color = Color::rgb_i(227, 183, 61);
    let border_color: Color = Color::rgb_i(140, 55, 96);
    let black_color = Color::rgba(0.0, 0.0, 0.0, 1.0);

    let s = nvg::Extent::new(SQUARE_SIZE as f32, SQUARE_SIZE as f32);

    for row in 0..universe.height {
        for col in 0..universe.width {
            let cell_state = universe.get_cell_state(row, col);

            ctx.begin_path();
            ctx.stroke_paint(border_color);
            let p = nvg::Point::new((col * SQUARE_SIZE) as f32, (row * SQUARE_SIZE) as f32);
            ctx.rect(nvg::Rect::new(p, s));
            if cell_state == Cell::Alive {
                ctx.fill_paint(orange_color)
            } else {
                 ctx.fill_paint(black_color);
            }
            ctx.fill()?;
        }
    }


    Ok(())
}

fn cursor_moved(_x: f32, _y: f32) {

}

fn main() {
    let mut el = glutin::event_loop::EventLoop::new();
    // let wb = glutin::window::WindowBuilder::new().with_dimensions(glutin::dpi::LogicalSize::new(1024.0, 768.0));
    let wb = glutin::window::WindowBuilder::new().with_fullscreen(Some(glutin::window::Fullscreen::Borderless(el.primary_monitor())));
    let windowed_context = glutin::ContextBuilder::new().build_windowed(wb, &el).unwrap();
    let windowed_context = unsafe { windowed_context.make_current().unwrap() };
    gl::load_with(|p| windowed_context.get_proc_address(p) as *const _);

    let renderer = nvg_gl::Renderer::create().unwrap();
    let mut context = nvg::Context::create(renderer).unwrap();

    init(&mut context).unwrap();

    let mut total_frames = 0;
    let start_time = Instant::now();
    let mut last_time = Instant::now();
    
    let inner_size = windowed_context.window().inner_size();
    let mut universe: Universe = Universe::new(inner_size.width/SQUARE_SIZE, inner_size.height/SQUARE_SIZE);

    el.run(move |event, _, control_flow| {
        // println!("{:?}", event);
        *control_flow = glutin::event_loop::ControlFlow::Poll;

        match event {
            glutin::event::Event::LoopDestroyed => return,
            glutin::event::Event::WindowEvent { event, .. } => match event {
                glutin::event::WindowEvent::Resized(physical_size) => {
                    windowed_context.resize(physical_size);
                    universe = Universe::new(physical_size.width/SQUARE_SIZE, physical_size.height/SQUARE_SIZE);
                }
                glutin::event::WindowEvent::CloseRequested => {
                    *control_flow = glutin::event_loop::ControlFlow::Exit
                }
                glutin::event::WindowEvent::KeyboardInput {
                    input:
                        glutin::event::KeyboardInput {
                            virtual_keycode: Some(virtual_code),
                            state,
                            ..
                        },
                    ..
                } => match (virtual_code, state) {
                    (glutin::event::VirtualKeyCode::Escape, _) => *control_flow = glutin::event_loop::ControlFlow::Exit,
                    (glutin::event::VirtualKeyCode::F, glutin::event::ElementState::Pressed) => {
                    
                        if !windowed_context.window().fullscreen().is_some() {
                            windowed_context.window().set_fullscreen(Some(glutin::window::Fullscreen::Borderless(windowed_context.window().primary_monitor())));
                        } else {
                            windowed_context.window().set_fullscreen(None);
                        }
                    }
                    _ => (),
                },                    
                _ => (),
            },
            glutin::event::Event::RedrawRequested(_) => {


            }
            _ => (),
        }

        let time_diff = (Instant::now() - last_time).as_secs_f32();
        if time_diff > 0.1 {
            // println!("Tick {:?}\n", (Instant::now() - last_time));
            last_time = Instant::now();
            universe.tick();
            // universe.debug_print();
        }

        let size = windowed_context.window().inner_size();
        let device_pixel_ratio = windowed_context.window().scale_factor() as f32;

        
        unsafe {
            gl::Viewport(
                0,
                0,
                (size.width as f32) as i32,
                (size.height as f32) as i32,
            );
            gl::ClearColor(0.0, 0.0, 0.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT | gl::STENCIL_BUFFER_BIT);
        }

        context
            .begin_frame(
                nvg::Extent {
                    width: size.width as f32,
                    height: size.height as f32,
                },
                device_pixel_ratio,
            )
            .unwrap();

        if universe.dirty {
            context.save();
            update(&universe, size.width as f32, size.height as f32, &mut context)
                .unwrap();
            context.restore();
        }

        total_frames += 1;
        let fps = (total_frames as f32) / (Instant::now() - start_time).as_secs_f32();
        context.fill_paint(Color::rgb(1.0, 0.0, 0.0));
        context.font("roboto");
        context.font_size(50.0);
        context.begin_path();
        context.text_align(Align::TOP | Align::LEFT);
        context.text((20, 10), format!("FPS: {:.2}", fps)).unwrap();
        context.fill().unwrap();

        context.end_frame().unwrap();
        windowed_context.swap_buffers().unwrap();        

    });
}



