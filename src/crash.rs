//! Crash reporting system for Trapiron.

use std::pin::pin;

/// Generates a crash report and outputs it from the program.
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

mod context {

    use std::cell::UnsafeCell;
    use std::marker::PhantomPinned;
    use std::pin::Pin;

    trait EntryLike {
        fn evaluate(&mut self) -> (&'static str, String);
    }

    struct Context {
        entries: Option<Vec<*mut dyn EntryLike>>,
    }

    impl Context {
        fn new() -> Self {
            Self {
                entries: Some(Vec::new()),
            }
        }
    }

    thread_local! {
        static CONTEXT: UnsafeCell<Context> = UnsafeCell::new(Context::new());
    }

    pub struct Entry<F>
    where
        F: FnOnce() -> String,
    {
        key: &'static str,
        value_supplier: Option<F>,
        _pin: PhantomPinned,
    }

    impl<F> Entry<F>
    where
        F: FnOnce() -> String,
    {
        pub fn from(key: &'static str, value_supplier: F) -> Self {
            Self {
                key,
                value_supplier: Some(value_supplier),
                _pin: Default::default(),
            }
        }

        pub fn publish(self: Pin<&mut Self>) {
            unsafe {
                let context: &mut Context = CONTEXT.with(|c| &mut *c.get());
                if let Some(entries) = &mut context.entries {
                    let x: &mut Self = self.get_unchecked_mut();
                    entries.push(x as *mut Self as *mut dyn EntryLike);
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
        fn drop(&mut self) {
            unsafe {
                let context: &mut Context = CONTEXT.with(|c| &mut *c.get());
                if let Some(entries) = &mut context.entries {
                    let me = &*self as *const Self as usize;
                    let popped = entries.pop().map(|p| p as *const Self as usize);
                    debug_assert_eq!(popped, Some(me));
                }
            }
        }
    }

    pub fn take() -> Vec<(&'static str, String)> {
        let entries = unsafe {
            let context: &mut Context = CONTEXT.with(|c| &mut *c.get());
            context.entries.take().unwrap_or_else(|| Vec::new())
        };

        let mut result: Vec<(&'static str, String)> = Vec::with_capacity(entries.len());

        for entry_ptr in entries.into_iter() {
            let entry: &mut dyn EntryLike = unsafe { &mut *entry_ptr };

            // Executing the value_supplier of an Entry cannot drop any Entry that was published
            // after it
            result.push(entry.evaluate());
        }

        result
    }
}

pub trait Reportable {
    fn present(&self) -> String;
}

impl<T> Reportable for T
where
    T: std::fmt::Debug,
{
    fn present(&self) -> String {
        format!("{:?}", self)
    }
}

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

pub fn setup_panic_hook() {
    std::panic::set_hook(Box::new(|info| {
        report_crash(&format!("{info}"));
    }))
}
