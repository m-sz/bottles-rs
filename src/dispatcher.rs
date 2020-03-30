//! Holds the Dispatcher struct which is the heart of the message mechanism.
//!
use std::collections::HashMap;
use std::any::TypeId;
use std::rc::Rc;

type UntypedCallback = Box<dyn FnMut(*const ())>;

/// Dispatches messages among subscribers
///
/// A subscriber is a closure which takes exactly one argument: `Rc<T>` where `T` is the type of
/// message to receive.
///
/// [`Dispatcher`] is type safe by having a mapping of the subscriber's expected type TypeId's and
/// the closures handling them.
///
pub struct Dispatcher {
    map: HashMap<TypeId, Vec<UntypedCallback>>,
}

impl Dispatcher {
    /// Creates a new [`Dispatcher`].
    pub fn new() -> Self {
        Self {
            map: HashMap::new()
        }
    }

    /// Registers the type to be available for subscribing
    ///
    /// It is a design choice that subscribing for a message of type that has not yet been
    /// registered will result in a panic (error in the future).
    ///
    /// # Safety
    /// This function uses unsafe internally, because of the need of mapping a `Rc<T>` into a
    /// `*const ()` for homogeneous storage.
    ///
    /// The transmutation fo `*const ()` is done in a closure that knows the target type `Rc<T>` and
    /// the usage is *safe* because the [Dispatcher] type holds a mapping between a `TypeId` and
    /// the corresponding closure which deals with the type itself.
    ///
    pub fn register<T: 'static>(&mut self)
    {
        self.map.entry(TypeId::of::<T>()).or_default();
    }

    /// Adds a subscriber for a message of type `T`.
    /// Whenever a value of type `Rc<T>` will be dispatched the provided callback will be executed.
    ///
    /// # Example
    /// ```
    /// # use std::rc::Rc;
    /// use bottles::dispatcher::Dispatcher;
    ///
    /// struct Message {}
    ///
    /// let mut dispatcher = Dispatcher::new();
    /// dispatcher.register::<Message>();
    /// dispatcher.subscribe(|message: Rc<Message>| {
    ///     // react to the message
    /// });
    pub fn subscribe<F, T>(&mut self, mut f: F)
        where
            F: FnMut(Rc<T>) + 'static,
            T: 'static
    {
        match self.map.get_mut(&TypeId::of::<T>()) {
            None => panic!("Can't subscribe to a message which is not registered!"),
            Some(subs) => subs
        };

        let wrapped = move |msg: *const ()| {
            let typed = unsafe { Rc::from_raw(msg as *const T) };
            (f)(typed)
        };

        self.map.get_mut(&TypeId::of::<T>()).unwrap().push(Box::new(wrapped));
    }

    /// Dispatches the message of value `T` to all of the subscribers
    ///
    /// # Safety
    /// This method casts untyped *const () message received internally into the expected type.
    /// It is safe to be called because of the mapping that ensures the correct messages are
    /// distributed to subscribers expecting them.
    pub fn dispatch<T: 'static>(&mut self, msg: Rc<T>) {
        let subscribers = self.map.get_mut(&TypeId::of::<T>())
            .expect("Can not dispatch a message which has not been registered");

        for subscriber in subscribers.iter_mut() {
            let untyped = unsafe { Rc::into_raw(Rc::clone(&msg)) };
            (subscriber)(untyped as *const ());
        }
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;
    use std::cell::RefCell;
    use super::{Dispatcher};

    struct Empty {}

    struct Greeting {
        greeting: String
    }

    struct Farewell {
        farawell: String
    }

    struct Context {
        answer: i32,
        called: bool,
    }

    fn pop(msg: Rc<Greeting>) {
        println!("Greeting {}", msg.greeting);
        assert_eq!(&msg.greeting, "Hello, World!");
    }

    #[test]
    fn test_basic()
    {
        let mut dispatcher = Dispatcher::new();
        dispatcher.register::<Greeting>();

        let called = Rc::new(RefCell::new(false));
        {
            let called = called.clone();
            let pop = move |msg| {
                *called.borrow_mut() = true;
                pop(msg);
            };
            dispatcher.subscribe(pop);
        }

        let message = Greeting {
            greeting: "Hello, World!".to_string()
        };

        dispatcher.dispatch(Rc::new(message));

        assert_eq!(*called.borrow(), true);
    }

    #[test]
    fn test_multiple_subscribers()
    {
        let mut dispatcher = Dispatcher::new();
        dispatcher.register::<Greeting>();

        let counter = Rc::new(RefCell::new(0));
        for _ in 0..2 {
            let counter = Rc::clone(&counter);

            dispatcher.subscribe(move |msg: Rc<Greeting>| {
                assert_eq!(msg.greeting, "Hello, World!");
                *counter.borrow_mut() += 1;
            });
        }

        dispatcher.dispatch(Rc::new(Greeting { greeting: "Hello, World!".to_string() }));

        assert_eq!(*counter.borrow(), 2);
    }
}
