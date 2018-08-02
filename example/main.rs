extern crate signal_handler
extern crate tokio_core;


//The only thing this sample implementation does is print out the some text when a signal is received
struct SignalPollPrinter;
impl Handler for SignalPollPrinter{
    fn ready_some(&self, signal: i32)->Poll<Option<i32>, std::io::Error> where Self: Sized{
        println!("SignalStream returned with a result: {}", signal);
        Ok(Async::Ready(Some(signal)))
    }
    fn not_ready(&self)->Poll<Option<i32>, std::io::Error> where Self: Sized{
        println!("SignalStream got nothing for now");
        Ok(Async::NotReady)
    }
    fn ready_none(&self)->Poll<Option<i32>, std::io::Error> where Self: Sized{
        println!("SignalStream ended");
        Ok(Async::Ready(None))
    }
    fn err(&self, e: std::io::Error)->Poll<Option<i32>, std::io::Error> where Self: Sized{
        println!("SignalStream gave back error: {}", e);
        Err(e)
    }
}

// This will handle every SIGINT signal 
// (ctrl+c or kill -2 <process> or other in this example)

fn main() {
    let mut core = tokio_core::reactor::Core::new().unwrap();
    let signal_tester = SignalHandler::new(2, SignalPollPrinter);
    core.run(signal_tester.for_each(|_|futures::future::ok(())));
}
