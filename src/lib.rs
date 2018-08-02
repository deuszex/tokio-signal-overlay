extern crate futures;
extern crate tokio_signal;

use futures::Poll;
use futures::Async;
use futures::Future;
use futures::Stream;

use tokio_signal::unix::Signal;

pub type BoxedSignalFuture = 
    std::boxed::Box< futures::Future<Error=std::io::Error, 
                                     Item=tokio_signal::unix::Signal> 
                     + std::marker::Send>;
pub type SignalStream = futures::FlattenStream<BoxedSignalFuture>;
pub type SignalPoll = Poll<tokio_signal::unix::Signal, std::io::Error>;

/// Callout to the tokio_signal crate
pub fn signal_recv(sig : i32)-> SignalStream{
    Signal::new(sig).flatten_stream()
}

/// This trait exists to give place to the signal handling. The implementation of this is used in the SignalHandler.
trait Handler {
    /// This function is called when the Signal Stream returns with a value
    fn ready_some(&self, signal: i32)->Poll<Option<i32>, std::io::Error> where Self: Sized;
    /// This function is called when the Signal Stream has nothing to return now
    fn not_ready(&self)->Poll<Option<i32>, std::io::Error> where Self: Sized;
    /// This function is called when the Signal Stream was closed down without an error
    fn ready_none(&self)->Poll<Option<i32>, std::io::Error> where Self: Sized;
    /// This function is called when the Signal Stream returns with an error
    fn err(&self, e: std::io::Error)->Poll<Option<i32>, std::io::Error> where Self: Sized;
}

/// This struct is the actual SignalHandler that is implemented as a Stream so it can be repeatedly polled
struct SignalHandler<T: Handler>{
    sig : i32,
    sig_stream : SignalStream,
    handler : T
}

impl<T: Handler> SignalHandler<T>{
    pub fn new(sig : i32, handler: T)->Self{
        Self{
            sig : sig,
            sig_stream : signal_recv(sig),
            handler: handler
        }
    }
    /// This is the main body of the SignalHandler, 
    /// that matches the returned poll from the SignalStream to the functions 
    /// implemented on the Handler trait
    pub fn handle_signal(&self, poll: Poll<Option<i32>, std::io::Error> )-> Poll<Option<i32>, std::io::Error> {
        match poll{
            Ok(Async::Ready(Some(s)))=>{
                return self.handler.ready_some(s);
            }
            Ok(Async::NotReady)=>{
                return self.handler.not_ready();                             
            }
            Ok(Async::Ready(None))=>{
                return self.handler.ready_none();
            }
            Err(e)=>{
                return self.handler.err(e);
            }   
        }
    }
}

impl<T: Handler> Stream for SignalHandler<T>{
    type Item = i32;
    type Error = std::io::Error;
    fn poll(&mut self)->Poll<Option<Self::Item>, Self::Error>{
        let poll = self.sig_stream.poll();
        self.handle_signal(poll)
    }
}

