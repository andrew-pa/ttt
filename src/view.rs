use skia_safe::Canvas;
use winit::event::WindowEvent;

use crate::presenter::Presenter;


pub struct View {
    presenter: Presenter
}

impl View {
    pub fn new(presenter: Presenter) -> View {
        View {
            presenter
        }
    }

    pub fn draw(&self, canvas: &mut Canvas) {
    }

    pub fn process_event(&mut self, e: WindowEvent) {
    }
}
