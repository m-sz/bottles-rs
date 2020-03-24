use std::{
    any::TypeId,
    cell::RefCell,
    collections::HashMap,
    rc::Rc,
};
use crate::dispatcher::Dispatcher;

type UntypedQueueCallback<T> = Box<dyn FnMut(&mut T, *const ())>;

pub struct Queue<T>
{
    map: HashMap<TypeId, Vec<UntypedQueueCallback<T>>>,
    queues: HashMap<TypeId, Rc<RefCell<Vec<*const ()>>>>,
}

impl<T> Queue<T> {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
            queues: HashMap::new()
        }
    }

    pub fn register<M: 'static>(&mut self, dispatcher: &mut Dispatcher) {
        dispatcher.register::<M>();
        self.map.entry(TypeId::of::<M>()).or_default();
        self.queues.entry(TypeId::of::<M>()).or_default();
    }

    pub fn subscribe<F, M>(&mut self, dispatcher: &mut Dispatcher, mut f: F)
        where
            F: FnMut(&mut T, Rc<M>) + 'static,
            M: 'static
    {
        let queue = self.queues.get(&TypeId::of::<M>()).unwrap().clone();
        let enqueue = move |msg: Rc<M>| {
            let mut queue = queue.borrow_mut();
            queue.push(unsafe { Rc::into_raw(msg) } as *const ());
        };

        let wrapped = move |ctx: &mut T, msg: *const ()| {
            let msg = unsafe { Rc::from_raw(msg as *const M) };
            (f)(ctx, msg);
        };

        dispatcher.subscribe(enqueue);
        self.map.get_mut(&TypeId::of::<M>())
            .unwrap()
            .push(Box::new(wrapped));
    }

    pub fn poll(&mut self, context: &mut T) {
        for (t, queue) in self.queues.iter_mut() {
            let queue = queue.borrow_mut();
            if queue.len() == 0 { continue; }

            for &item in queue.iter() {
                for subscriber in self.map.get_mut(t).unwrap() {
                    (subscriber)(context, item);
                }
            }

        }
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;
    use super::Queue;
    use crate::dispatcher::Dispatcher;

    struct Greeting { greeting: String }
    struct Farawell { farawell: String }
    struct Context { answer: i32, called: bool }

    fn context_pop(ctx: &mut Context, msg: Rc<Greeting>) {
        assert_eq!(ctx.answer, 42);
        assert_eq!(&msg.greeting, "Hello, World!");

        ctx.called = true;
    }

    #[test]
    fn test_wrapping()
    {
        let mut dispatcher = Dispatcher::new();
        let mut wrapping = Queue::<Context>::new();
        let mut ctx = Context {
            answer: 42,
            called: false,
        };

        let message = Greeting {
            greeting: "Hello, World!".to_string()
        };

        wrapping.register::<Greeting>(&mut dispatcher);
        wrapping.subscribe(&mut dispatcher, context_pop);

        dispatcher.dispatch(Rc::new(message));
        wrapping.poll(&mut ctx);

        assert_eq!(ctx.called, true);
    }

    #[test]
    fn test_queue_sink()
    {
        let mut dispatcher_a = Dispatcher::new();
        let mut dispatcher_b = Dispatcher::new();

        dispatcher_a.register::<Greeting>();
        dispatcher_b.register::<Farawell>();

        let mut queue = Queue::<()>::new();
        queue.register::<Greeting>(&mut dispatcher_a);
        queue.register::<Farawell>(&mut dispatcher_b);

        let handle_greeting = |_: &mut (), msg: Rc<Greeting>| {
            assert_eq!(msg.greeting, "Hello, World!");
        };
        let handle_farawell = |_: &mut (), msg: Rc<Farawell>| {
            assert_eq!(msg.farawell, "Goodbye!");
        };

        queue.subscribe(&mut dispatcher_a, handle_greeting);
        queue.subscribe(&mut dispatcher_b, handle_farawell);

        dispatcher_a.dispatch(Rc::new(Greeting {
            greeting: "Hello, World!".to_string()
        }));
        dispatcher_b.dispatch(Rc::new(Farawell {
            farawell: "Goodbye!".to_string()
        }));

        queue.poll(&mut ());
    }

}