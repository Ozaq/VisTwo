use std::fmt::Debug;
use winit::event::ElementState;
use winit::event::Event;
use winit::event::KeyboardInput;
use winit::event::VirtualKeyCode;
use winit::event::WindowEvent;

#[derive(Debug)]
pub struct KeyMap {
    pressed_keys: Vec<VirtualKeyCode>,
}

impl Default for KeyMap {
    fn default() -> Self {
        Self::new()
    }
}

impl KeyMap {
    pub fn new() -> Self {
        Self {
            pressed_keys: Vec::new(),
        }
    }

    pub fn begin_frame(&mut self) {
        self.pressed_keys.clear();
    }

    pub fn handle_event<T>(&mut self, evt: &Event<T>)
    where
        T: Debug,
    {
        if let Event::WindowEvent {
            event:
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            virtual_keycode: Some(key),
                            state: ElementState::Pressed,
                            ..
                        },
                    ..
                },
            ..
        } = evt
        {
            self.pressed_keys.push(*key);
        };
    }
}
