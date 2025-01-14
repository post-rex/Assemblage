use cgmath::Vector3;
use winit::event::{ElementState, KeyboardInput, VirtualKeyCode, WindowEvent};

use crate::rendering::camera::Camera;

pub struct CameraController {
    speed: f32,
    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool,
    is_up_pressed: bool,
    is_down_pressed: bool,
    is_pitch_up_pressed: bool,
    is_pitch_down_pressed: bool,
}

impl CameraController {
    pub fn new(speed: f32) -> Self {
        Self {
            speed,
            is_forward_pressed: false,
            is_backward_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
            is_up_pressed: false,
            is_down_pressed: false,
            is_pitch_up_pressed: false,
            is_pitch_down_pressed: false,
        }
    }

    pub fn process_events(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state,
                        virtual_keycode: Some(keycode),
                        ..
                    },
                ..
            } => {
                let is_pressed = *state == ElementState::Pressed;
                match keycode {
                    VirtualKeyCode::W | VirtualKeyCode::Up => {
                        self.is_forward_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::A | VirtualKeyCode::Left => {
                        self.is_left_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::S | VirtualKeyCode::Down => {
                        self.is_backward_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::D | VirtualKeyCode::Right => {
                        self.is_right_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::E => {
                        self.is_pitch_down_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::Q => {
                        self.is_pitch_up_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::Space => {
                        self.is_up_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::LShift => {
                        self.is_down_pressed = is_pressed;
                        true
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }

    pub fn update_camera(&self, camera: &mut Camera) {
        use cgmath::InnerSpace;
        let forward = camera.target - camera.eye;
        let forward_norm = forward.normalize();
        let forward_mag = forward.magnitude();

        // Prevents glitching when camera gets too close to the
        // center of the scene.
        if self.is_forward_pressed && forward_mag > self.speed {
            camera.eye += forward_norm * self.speed;
            camera.target += forward_norm * self.speed;
        }
        if self.is_backward_pressed {
            camera.eye -= forward_norm * self.speed;
            camera.target -= forward_norm * self.speed;
        }

        if self.is_up_pressed {
            camera.eye += Vector3::unit_y() * self.speed;
            camera.target += Vector3::unit_y() * self.speed;
        }
        if self.is_down_pressed {
            camera.eye -= Vector3::unit_y() * self.speed;
            camera.target -= Vector3::unit_y() * self.speed;
        }

        let right = forward_norm.cross(camera.up) * 0.1;

        // Redo radius calc in case the fowrard/backward is pressed.
        let forward = camera.target - camera.eye;
        let forward_mag = forward.magnitude();

        if self.is_right_pressed {
            camera.eye = camera.target
                - (forward + right * self.speed / forward_mag).normalize() * forward_mag;
        }
        if self.is_left_pressed {
            camera.eye = camera.target
                - (forward - right * self.speed / forward_mag).normalize() * forward_mag;
        }
        if self.is_pitch_up_pressed {
            camera.eye = camera.target
                - (forward + (camera.up * 0.1) * self.speed / forward_mag).normalize()
                    * forward_mag;
        }
        if self.is_pitch_down_pressed {
            camera.eye = camera.target
                - (forward - (camera.up * 0.1) * self.speed / forward_mag).normalize()
                    * forward_mag;
        }
    }
}
