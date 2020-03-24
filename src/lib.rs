#![deny(missing_docs)]

//! bottles
//! ===
//! An enum-less typed message passing mechanism
//!
//! This crate allows to put a message in form of any 'static type in a bottles, which then
//! can be dispatched to subscribers of that type.
//!
//! There are 2 main types in this crate: [`Dispatcher`] and [`Queue`]
//!
//! ### Dispatcher
//! A simple mapping between a types and a list of subscribers to a message of that type.
//! The subscriber is a closure which takes a single argument of type `Rc<T>` which will be called
//! whenever a message with that type is dispatched.
//!
//! Most often it is cumbersome to react to messages without any *context*, one would have to rely
//! on `Rc<RefCell<Context>>` capturing in the closure in order to mutate the outside world.
//! The easiest solution provided is to use a [`Queue`] to act as an intermediate component between
//! the [`Dispatcher`] and the callback itself.
//!
//! ### Queue
//!
//! Queue allows one to collect messages from one or several dispatchers and queue them internally
//! to be dispatched on a call to the [`Queue.poll`] method which *allows* for providing a "context"
//! value to all subscribers.
//!
//! The Queue subscriber is a closure taking exactly 2 arguments: a `&mut` reference to the context
//! variable and a `Rc<T>` for message of type `T`
//!
//! ### Examples
//!
//! Send a greeting to a subscriber
//! ```
//! use std::rc::Rc;
//! use bottles::Dispatcher;
//!
//! struct Greeting {
//!     greeting: String
//! }
//!
//! fn callback(msg: Rc<Greeting>) {
//!     println!("Greeting: {}", msg.greeting);
//! }
//!
//!     let mut dispatcher = Dispatcher::new();
//!
//!     // Every message type has to be registered explicitly
//!     // It is a design choice to have a error when subscribing for a message type which is
//!     // not registered by the particular Dispatcher, to error out as quickly as possible
//!     dispatcher.register::<Greeting>();
//!
//!     dispatcher.subscribe(callback);
//!
//!     dispatcher.dispatch(Rc::new(Greeting { greeting: "Hello there!".to_string() }));
//!
//! ```
//!
//! Create a callback which has a mutable context
//! ```
//! use std::rc::Rc;
//! use bottles::{Dispatcher, Queue};
//!
//! struct Add(i32);
//!
//! struct Context {
//!     state: i32
//! }
//!
//! fn add(ctx: &mut Context, msg: Rc<Add>) {
//!     ctx.state += msg.0;
//! }
//!
//! let mut dispatcher = Dispatcher::new();
//! let mut queue = Queue::new();
//!
//! queue.register::<Add>(&mut dispatcher);
//!
//! queue.subscribe(&mut dispatcher, add);
//!
//! // `queue` will receive the message and enqueue it for retrieval when polled.
//! dispatcher.dispatch(Rc::new(Add(42)));
//!
//! let mut context = Context {
//!     state: 0
//! };
//!
//! // Very important: Queue has works by being polled, because this allows to pass a reference
//! // to the context without resorting to any interior mutability patterns.
//! queue.poll(&mut context);
//!
//! assert_eq!(context.state, 42);
//! ```
//!
//! [`Dispatcher`]: dispatcher/struct.Dispatcher.html
//! [`Queue`]: queue/struct.Queue.html
//! [`Queue.poll`]: queue/struct.Queue.html#method.poll

///
pub mod dispatcher;
///
pub mod queue;

///
pub use {dispatcher::Dispatcher, queue::Queue};

///
pub mod prelude {
    pub use crate::dispatcher::Dispatcher;
    pub use crate::queue::Queue;
}


