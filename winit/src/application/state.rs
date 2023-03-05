use crate::application::{self, StyleSheet as _};
use crate::conversion;
use crate::core;
use crate::core::{Color, Point, Size};
use crate::graphics::Viewport;
use crate::runtime::Debug;
use crate::Application;

use std::marker::PhantomData;
use winit::event::{Touch, WindowEvent};
use winit::window::Window;

/// The state of a windowed [`Application`].
#[allow(missing_debug_implementations)]
pub struct State<A: Application>
where
    <A::Renderer as core::Renderer>::Theme: application::StyleSheet,
{
    title: String,
    scale_factor: f64,
    viewport: Viewport,
    viewport_version: usize,
    cursor_position: winit::dpi::PhysicalPosition<f64>,
    modifiers: winit::event::ModifiersState,
    theme: <A::Renderer as core::Renderer>::Theme,
    appearance: application::Appearance,
    application: PhantomData<A>,
}

impl<A: Application> State<A>
where
    <A::Renderer as core::Renderer>::Theme: application::StyleSheet,
{
    /// Creates a new [`State`] for the provided [`Application`] and window.
    pub fn new(application: &A, window: &Window) -> Self {
        let title = application.title();
        let scale_factor = application.scale_factor();
        let theme = application.theme();
        let appearance = theme.appearance(&application.style());

        let viewport = {
            let physical_size = window.inner_size();

            Viewport::with_physical_size(
                Size::new(physical_size.width, physical_size.height),
                window.scale_factor() * scale_factor,
            )
        };

        Self {
            title,
            scale_factor,
            viewport,
            viewport_version: 0,
            // TODO: Encode cursor availability in the type-system
            cursor_position: winit::dpi::PhysicalPosition::new(-1.0, -1.0),
            modifiers: winit::event::ModifiersState::default(),
            theme,
            appearance,
            application: PhantomData,
        }
    }

    /// Returns the current [`Viewport`] of the [`State`].
    pub fn viewport(&self) -> &Viewport {
        &self.viewport
    }

    /// Returns the version of the [`Viewport`] of the [`State`].
    ///
    /// The version is incremented every time the [`Viewport`] changes.
    pub fn viewport_version(&self) -> usize {
        self.viewport_version
    }

    /// Returns the physical [`Size`] of the [`Viewport`] of the [`State`].
    pub fn physical_size(&self) -> Size<u32> {
        self.viewport.physical_size()
    }

    /// Returns the logical [`Size`] of the [`Viewport`] of the [`State`].
    pub fn logical_size(&self) -> Size<f32> {
        self.viewport.logical_size()
    }

    /// Returns the current scale factor of the [`Viewport`] of the [`State`].
    pub fn scale_factor(&self) -> f64 {
        self.viewport.scale_factor()
    }

    /// Returns the current cursor position of the [`State`].
    pub fn cursor_position(&self) -> Point {
        conversion::cursor_position(
            self.cursor_position,
            self.viewport.scale_factor(),
        )
    }

    /// Returns the current keyboard modifiers of the [`State`].
    pub fn modifiers(&self) -> winit::event::ModifiersState {
        self.modifiers
    }

    /// Returns the current theme of the [`State`].
    pub fn theme(&self) -> &<A::Renderer as core::Renderer>::Theme {
        &self.theme
    }

    /// Returns the current background [`Color`] of the [`State`].
    pub fn background_color(&self) -> Color {
        self.appearance.background_color
    }

    /// Returns the current text [`Color`] of the [`State`].
    pub fn text_color(&self) -> Color {
        self.appearance.text_color
    }

    /// Processes the provided window event and updates the [`State`]
    /// accordingly.
    pub fn update(
        &mut self,
        window: &Window,
        event: &WindowEvent<'_>,
        _debug: &mut Debug,
    ) {
        match event {
            WindowEvent::Resized(new_size) => {
                let size = Size::new(new_size.width, new_size.height);

                self.viewport = Viewport::with_physical_size(
                    size,
                    window.scale_factor() * self.scale_factor,
                );

                self.viewport_version = self.viewport_version.wrapping_add(1);
            }
            WindowEvent::ScaleFactorChanged {
                scale_factor: new_scale_factor,
                new_inner_size,
            } => {
                let size =
                    Size::new(new_inner_size.width, new_inner_size.height);

                self.viewport = Viewport::with_physical_size(
                    size,
                    new_scale_factor * self.scale_factor,
                );

                self.viewport_version = self.viewport_version.wrapping_add(1);
            }
            WindowEvent::CursorMoved { position, .. }
            | WindowEvent::Touch(Touch {
                location: position, ..
            }) => {
                self.cursor_position = *position;
            }
            WindowEvent::CursorLeft { .. } => {
                // TODO: Encode cursor availability in the type-system
                self.cursor_position =
                    winit::dpi::PhysicalPosition::new(-1.0, -1.0);
            }
            WindowEvent::ModifiersChanged(new_modifiers) => {
                self.modifiers = *new_modifiers;
            }
            #[cfg(feature = "debug")]
            WindowEvent::KeyboardInput {
                input:
                    winit::event::KeyboardInput {
                        virtual_keycode: Some(winit::event::VirtualKeyCode::F12),
                        state: winit::event::ElementState::Pressed,
                        ..
                    },
                ..
            } => _debug.toggle(),
            _ => {}
        }
    }

    /// Synchronizes the [`State`] with its [`Application`] and its respective
    /// window.
    ///
    /// Normally an [`Application`] should be synchronized with its [`State`]
    /// and window after calling [`Application::update`].
    ///
    /// [`Application::update`]: crate::Program::update
    pub fn synchronize(&mut self, application: &A, window: &Window) {
        // Update window title
        let new_title = application.title();

        if self.title != new_title {
            window.set_title(&new_title);

            self.title = new_title;
        }

        // Update scale factor and size
        let new_scale_factor = application.scale_factor();
        let new_size = window.inner_size();
        let current_size = self.viewport.physical_size();

        if self.scale_factor != new_scale_factor
            || (current_size.width, current_size.height)
                != (new_size.width, new_size.height)
        {
            self.viewport = Viewport::with_physical_size(
                Size::new(new_size.width, new_size.height),
                window.scale_factor() * new_scale_factor,
            );
            self.viewport_version = self.viewport_version.wrapping_add(1);

            self.scale_factor = new_scale_factor;
        }

        // Update theme and appearance
        self.theme = application.theme();
        self.appearance = self.theme.appearance(&application.style());
    }
}
