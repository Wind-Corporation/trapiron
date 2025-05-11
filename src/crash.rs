//! Crash reporting system for Trapiron.
//!
//! Crashes are Rust panics with user-friendly information. The main feature of crashes is
//! the _crash context_.
//!
//! # Crash context
//!
//! Crash context is a thread-local stack of context entries. Each entry can provide some
//! information on the details of the operation being performed in case of a panic.
//!
//! For example, a crash context entry may contain information on the block currently being updated.
//! The code responsible for updating all blocks should push the entry before calling the update
//! routine, then pop it back when the routine completes without panicking. This way, if the routine
//! panics, the crash context can be inspected to learn what block caused the crash.
//!
//! Use [`with_context()`] to manage context entries. [`setup_panic_hook()`] should be called once
//! to install the panic handler.

use std::pin::pin;

/// Generates a crash report and outputs it from the program.
///
/// `message` is a human-readable description of the problem that caused the crash.
///
/// The report will contain the message and all current context entries. See [`with_context()`].
///
/// The effect of calling this function twice in the same thread is not specified. The format of
/// the crash report and how it is output are implementation details.
pub fn report_crash(message: &str) {
    let header = "==== Trapiron crash report ====\nTrapiron has crashed!";
    let footer = "====== Crash report end =======";
    let funny = {
        let options = [":(", ":\\", ":/", "o_o", ">:[", ":|"];
        options[message.len() % options.len()]
    };

    let context = context::take()
        .into_iter()
        .fold(String::new(), |s, (key, value)| {
            s + &format!("{}:\n    {}\n", key, value)
        });

    eprintln!(
        "{} {}\n\n{}\n\n{}\n\n{}",
        &header, &funny, &message, &context, &footer
    );
}

/// The unsafe implementation details of the crash context.
///
/// # Design considerations
///
/// Crash context must operate very efficiently when no panics occur. Instead of copying values
/// around, the context entries (string pointer and closure) are stored on the call stack as local
/// variables, and only a pointer to that data is pushed into a global vector.
///
/// Hopefully, this results in just a few processor instructions of overhead:
///   - **Add entry:** push name string pointer, push closure function pointer, push closure
///     captures, append stack pointer to context.
///   - **Remove entry:** decrement context size by one, drop closure captures.
///
/// If a panic does occur, the entries of the context are accessible via the stored pointers to data
/// in the call stack.
///
/// # Implementation
///
/// Crash context is a [`thread_local!`] [`Vec`] of pointers to entries.
///
/// The dynamic allocation of `Vec` is not a concern, since the size of an entry in this vector is
/// fixed (one pointer). The vector will likely grow to its final size within the first frame render
/// or update, after which memory will be reused. Furthermore, because the `Vec` is located in a
/// `thread_local!` variable, and its backing buffer will not regularly change, hopefully the entire
/// data structure will be cached by the CPU.
///
/// Note that because this code runs to handle panics, it should do its best not panic itself.
/// Whereever reasonable, panics are replaced with some sort of technically valid behavior.
///
/// ## Borrow Checker and Safety
///
/// Unfortunately, it is not possible to store safe references to the entry data in the vector. This
/// is because they must be popped on drop, and there is no convenient inbuilt mechanism for
/// expressing their lifetime in Rust.
///
/// After exhausing my own creativity, including exploring solutions where the entry data is owned
/// by the vector, I decided to go with carefully managed raw pointers instead.
///
/// ## Entries
///
/// [`Entry`](context::Entry) is the struct that owns the name string and the closure. As such, it
/// must be generic. To make handling it easier, a type-erased trait is used, `EntryLike`.
mod context {

    use std::cell::UnsafeCell;
    use std::marker::PhantomPinned;
    use std::pin::Pin;

    /// A type-erased [`Entry`].
    trait EntryLike {
        /// Invokes the value supplier to obtain the value and returns the name and the value.
        ///
        /// Since value supplier is an [`FnOnce`], this method should not be called more than once,
        /// although doing so is technically safe.
        ///
        /// This method does not take ownership of `self` because it has to be callable via `&mut`.
        /// The entry objects are owned by [`with_context()`](super::with_context()), but
        /// `evaluate()` is called by the panic handler.
        fn evaluate(&mut self) -> (&'static str, String);
    }

    /// The crash context.
    ///
    /// This struct only contains the context stack for now, but it might get expanded with other
    /// fields in the future.
    struct Context {
        /// The vector of currently active crash context entries.
        ///
        /// The field is initialized with an empty vector on first access. It may be reset to `None`
        /// to indicate that no further entries may ever be pushed.
        entries: Option<Vec<*mut dyn EntryLike>>,
    }

    impl Context {
        /// Constructs a new crash context.
        fn new() -> Self {
            Self {
                entries: Some(Vec::new()),
            }
        }
    }

    thread_local! {
        /// The crash context of this thread.
        ///
        /// This is an `UnsafeCell` because we can guarantee that no recursion may occur.
        static CONTEXT: UnsafeCell<Context> = UnsafeCell::new(Context::new());
    }

    /// A crash context entry, i.e. the entry name and the value supplier closure.
    ///
    /// **Warning:** a constructed entry _must_ be [published](Entry::publish()) before it is
    /// **dropped!
    pub struct Entry<F>
    where
        F: FnOnce() -> String,
    {
        /// The name of this crash context entry.
        key: &'static str,

        /// A single-use supplier of the value of this context entry.
        ///
        /// The field is reset to `None` after the supplier is evaluated for the first time.
        value_supplier: Option<F>,

        /// Instances of this struct are intended to be pointed to by the crash context. Ensure here
        /// that the pointers cannot be invalidated as a safeguard.
        _pin: PhantomPinned,
    }

    impl<F> Entry<F>
    where
        F: FnOnce() -> String,
    {
        /// Constructs an entry by taking ownership of a name-supplier pair.
        ///
        /// **Warning:** a constructed value _must_ be [published](Entry::publish()) before it is
        /// **dropped!
        pub fn from(key: &'static str, value_supplier: F) -> Self {
            Self {
                key,
                value_supplier: Some(value_supplier),
                _pin: Default::default(),
            }
        }

        /// Pushes the memory address of this pinned entry into the crash context stack.
        ///
        /// See also [`drop()`].
        pub fn publish(self: Pin<&mut Self>) {
            unsafe {
                // CONTEXT safety: the execution tree of this block is known and it never
                // references CONTEXT again.
                let context: &mut Context = CONTEXT.with(|c| &mut *c.get());

                if let Some(entries) = &mut context.entries {
                    let x: &mut Self = self.get_unchecked_mut();
                    // SAFETY: We extend the lifetime of `Self` to `'static`, I don't know
                    // if this is sound.
                    let x = std::mem::transmute::<
                        *mut (dyn EntryLike + '_),
                        *mut (dyn EntryLike + 'static),
                    >(x);
                    entries.push(x);
                } else {
                    // Fail silently: crash context has been forever disabled for this thread
                }
            }
        }
    }

    impl<F> EntryLike for Entry<F>
    where
        F: FnOnce() -> String,
    {
        fn evaluate(&mut self) -> (&'static str, String) {
            (
                self.key,
                match self.value_supplier.take() {
                    Some(f) => (f)(),
                    None => "[Entry.evaluate called twice]".into(),
                },
            )
        }
    }

    impl<F> Drop for Entry<F>
    where
        F: FnOnce() -> String,
    {
        /// Pops the memory address of this entry from crash context stack.
        ///
        /// The entry _must_ have been [published](Entry::publish()) previously. A debug assertion
        /// should catch violations, but it is disabled in release builds.
        fn drop(&mut self) {
            unsafe {
                // CONTEXT safety: the execution tree of this block is known and it never
                // references CONTEXT again.
                let context: &mut Context = CONTEXT.with(|c| &mut *c.get());

                if let Some(entries) = &mut context.entries {
                    let me = &*self as *const Self as usize;

                    // The value being popped (therefore dropped) is a pointer, it has no user code.
                    //
                    // Note that we pop first, check if we popped the right thing later.
                    let popped = entries.pop().map(|p| p as *const Self as usize);

                    // Attempt to catch violations of the `Entry` contract. If something went wrong,
                    // crash everything here and now because the crash context is now in a bad
                    // state.
                    debug_assert_eq!(popped, Some(me));
                } else {
                    // The context has been consumed while this entry is active. Do nothing.
                }
            }
        }
    }

    /// Evaluates and returns context entries. Consumes and disables the context for this thread.
    ///
    /// Any user code attempting to push or pop context entries after this function starts will not
    /// alter the crash context.
    ///
    /// The side effect of disabling crash context is irreversible. Crash contexts in other threads
    /// are not affected. On second and further invocations this function returns an empty vector.
    pub fn take() -> Vec<(&'static str, String)> {
        let entries = unsafe {
            // CONTEXT safety: the execution tree of this block is known and it never
            // references CONTEXT again.
            let context: &mut Context = CONTEXT.with(|c| &mut *c.get());

            // Nothing is dropped here - no user code possible.
            //
            // If the context is already disabled, just pretend it was empty.
            context.entries.take().unwrap_or_else(|| Vec::new())
        };

        let mut result: Vec<(&'static str, String)> = Vec::with_capacity(entries.len());

        for entry_ptr in entries.into_iter() {
            // Safety: `Entry`s pop themselves on drop or panic. At no point does the vector
            // actually contain invalid pointers.
            //
            // Strictly speaking, if an `Entry` were to be dropped between the context being
            // disabled and this line executing for that entry, the drop() of the Entry wouldn't be
            // able to pop the pointer. However, no such drop may occur: with_context() ensures that
            // entries cannot be dropped by the closure invoked by evaluate().
            let entry: &mut dyn EntryLike = unsafe { &mut *entry_ptr };

            // User code invoked! Scary
            //
            // As noted above, it cannot possibly drop an Entry.
            result.push(entry.evaluate());
        }

        result
    }
}

/// A value that may turned itself into a [`String`] for crash reports.
pub trait Reportable {
    /// Converts this value into its string representation for crash reports.
    fn present(self) -> String;
}

impl<T> Reportable for T
where
    T: std::fmt::Debug,
{
    fn present(self) -> String {
        format!("{:?}", self)
    }
}

/// Executes an action with some context information in case it panics.
///
/// A crash report context item is pushed, the `action` is executed, and the item is popped.
/// The entry is a `(name, supplier)` pair, where `name` is a `&'static str`, and the `supplier` is
/// a closure that will be called if a panic occurs to supply the actual context data.
///
/// A panic hook needs to call [`report_crash()`] for this construct to have any effect.
/// See [`setup_panic_hook()`].
///
/// # Cost
///
/// This function has very little overhead if no panic occurs; dynamic memory allocations "almost
/// never" occur.
///
/// # Example
///
/// ```
/// let y = crash::with_context(("Value of x", || x), || flaky_function(x));
/// ```
pub fn with_context<V, S, F, R>(ctxt: (&'static str, S), action: F) -> R
where
    V: Reportable,
    S: FnOnce() -> V,
    F: FnOnce() -> R,
{
    let entry = pin!(context::Entry::from(ctxt.0, || (ctxt.1)().present()));
    entry.publish();
    action()
}

/// Installs the panic hook that calls [`report_crash()`].
pub fn setup_panic_hook() {
    std::panic::set_hook(Box::new(|info| {
        report_crash(&format!("{info}"));
    }))
}
