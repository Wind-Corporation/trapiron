//! The adapter to the less pleasant parts of glium, glutin and winit related to the winit
//! application lifecycle.

use glium::winit;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::WindowId;

use crate::gui::Application as UserApp;

/// Initializes the GUI, runs GUI main loop and shuts it down when exiting.
///
/// Due to the requirements of various underlying OS libraries, this function must be called in the
/// application main thread. It only returns after an exit is requested, and after GUI has shut
/// down.
///
/// In order to respond to user input, a [`UserApp`] object must be provided via `initializer`. This
/// object should directly or indirectly own all GUI-related resources.
///
/// A single [`Gui`](super::Gui) instance is created and later dropped by this function. It is only
/// available while this function runs, and it is only available in the thread of this function.
///
/// The exact order of events is as follows:
///   1. GUI initialization happens: window and OpenGL context are created.
///   2. `initializer` is executed. User logic may instantiate necessary resources, but blocking
///      operations should be deferred until the main loop to prevent UI freezes.
///   3. Main loop executes until an exit is requested. The object returned by `initializer`
///      receives events.
///   4. The object returned by `initializer` is dropped. User logic may release necessary
///      resources. Blocking operations should happen before GUI exits to prevent UI freezes.
///   5. GUI shuts down.
///   6. This function returns.
pub fn run<I, A>(initializer: I)
where
    I: FnOnce(&mut crate::gui::Gui) -> A,
    A: UserApp,
{
    let event_loop =
        winit::event_loop::EventLoop::new().expect("Could not create winit::EventLoop");

    let mut wapp = WinitApplication {
        state: ApplicationState::Ready(initializer),
    };

    event_loop
        .run_app(&mut wapp)
        .expect("General failure in winit::EventLoop");

    // Drops user Application, then drops Gui
    drop(wapp);
}

/// An enum that stores the evolving state of the [`WinitApplication`].
///
/// `A` is the concrete type of the UserApp, `I` is the type of the initializer from [`run()`].
///
/// Fun fact: the awkwardness of `ApplicationState` is the reason why [`run()`] accepts a
/// _constructor_ and not the `A` itself.
enum ApplicationState<A, I>
where
    I: FnOnce(&mut crate::gui::Gui) -> A,
    A: UserApp,
{
    /// The application is ready to complete GUI initialization and to construct the [`UserApp`].
    Ready(I),

    /// The application is undergoing GUI initialization.
    Initializing,

    /// The GUI initialization has completed, a [`UserApp`] instance exists.
    Running {
        /// The user application object.
        user_app: A,

        /// The [`Gui`](super::Gui) object.
        gui: crate::gui::Gui,
    },
}

/// An application object for `winit`; the entrypoint for GUI inputs and the owner of all
/// GUI-related resources.
///
/// This object is the owner of both the [`Gui`](super::Gui) object and the user application object.
///
/// The state of a `WinitApplication` is stored in a `state` enum. The object is constructed in the
/// [`Ready`](ApplicationState::Ready) state, then transitions
/// into the [`Running`](ApplicationState::Running) state during the first call to `resumed` as per
/// winit documentation.
///
/// `A` is the concrete type of the UserApp, `I` is the type of the initializer from [`run()`].
///
/// Not to be confused with [`crate::gui::Application`] (also known as `UserApp`).
struct WinitApplication<A, I>
where
    I: FnOnce(&mut crate::gui::Gui) -> A,
    A: UserApp,
{
    /// The state of this object.
    state: ApplicationState<A, I>,
}

impl<A, I> winit::application::ApplicationHandler<()> for WinitApplication<A, I>
where
    I: FnOnce(&mut crate::gui::Gui) -> A,
    A: UserApp,
{
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // Called by winit either immediately when the event loop starts, or whenever the host is
        // ready for window and OpenGL context creation.
        //
        // Suspended/Resumed events are ignored for all other purposes.

        use ApplicationState::*;

        // Transition Ready -> Initializing and extract the initializer, else return
        let initializer = if let Ready(_) = &mut self.state {
            let Ready(initializer) = std::mem::replace(&mut self.state, Initializing) else {
                unreachable!();
            };
            initializer
        } else {
            return;
        };

        // Perform GUI initialization
        let mut gui = crate::crash::with_context(("GUI setup phase", || "Backend"), || {
            super::Gui::new(event_loop)
        });

        // Construct user application object
        let user_app = crate::crash::with_context(("GUI setup phase", || "Application"), || {
            initializer(&mut gui)
        });

        // Transition to Running state
        self.state = Running { user_app, gui };
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let ApplicationState::Running {
            ref mut user_app,
            ref mut gui,
        } = self.state
        else {
            return;
        };

        if gui.backend.window.id() != window_id {
            return;
        }

        crate::crash::with_context(("Current winit (GUI) event", || &event), || {
            super::handle_event(gui, user_app, super::WinitEvent::Window(&event), event_loop);
        });
    }

    fn device_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) {
        let ApplicationState::Running {
            ref mut user_app,
            ref mut gui,
        } = self.state
        else {
            return;
        };

        if !gui.backend.window.has_focus() {
            return;
        }

        if !gui.cursor_captured() {
            return;
        }

        crate::crash::with_context(("Current winit (GUI) event", || &event), || {
            super::handle_event(gui, user_app, super::WinitEvent::Device(&event), event_loop);
        });
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let ApplicationState::Running { ref gui, .. } = self.state {
            gui.backend.window.request_redraw();
        }
    }
}
