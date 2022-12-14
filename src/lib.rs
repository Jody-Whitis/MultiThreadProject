#![allow(non_snake_case)] //Compiler to ignore warnings during build (project name was not snake cased)

use std::{ //Bring in namespaces of Rust's modules and standard library crates into scope for the class.
    //The other namespace "sync" and "thread" both are sub modules of std, a group of namespaces used instead as a short hand notation.
    //(instead of three or more different use statements).
    sync::{mpsc, Arc, Mutex}, //This module has three other sub-modules, bring these into scope in one statement.
    //These modules help synchronize multi-thread operations that can access and change mutable object values from more than one thread.
    //mpsc: Multi-producer, single consumer first in first out queue. Provides communication over channels. Sets up only one consumer or receiver with multiple callers.(multiple requests with a single destination).
    //Arc: atomically ref-counted pointer, used in multi-threaded apps to extend the lifetime of data until all threads are finished with it. Ensures thread safe operations.
    //Mutex: mutual exclusive lock to make sure no more than one thread accesses or alters data of an object. Only one thread at a time on data.
    thread, //thread: use of thread each with its own stack and local saved state. This brings in the thread type along with its operations.
};

pub struct ThreadPool{ //An object to have a pool of spawned threads. This will hold workers to perform work in the thread along with the sending of the job.
    workers: Vec<Worker>, //Holds vector of Worker structure that will store threads to perform operations.
    sender: Option<mpsc::Sender<Job>>, //Create a channel that will hold the sender of a Job that will be sent through the channel.
//Option either returns a object of Job to be sent with the Sender in a asynchronous channel (Some) or None (nothing to send)
}

type Job = Box<dyn FnOnce() + Send + 'static>; //Type alias to hold a pointer to a heap and a function pointer.
// A vary amount of jobs get saved in the heap as we may not know the exact amount of jobs/threads that are performing operations.
// We'll get a closure to be executed once as each job in a thread runs once before the thread is returned to the pool.

impl ThreadPool{ //The implementation of the pool that will hold sending, receiving and workers for each thread.
    pub fn new(size: usize) -> ThreadPool{ //Type of constructor that take a parameter of size (the number of max threads).
        assert!(size > 0); //Error handling macro to validate we have at least one thread running from the pool.

        let (sender,receiver)  = mpsc::channel(); //Tuple of an async channel of a sender and receiver of both will hold a Job structure.
        let receiver = Arc::new(Mutex::new(receiver)); //Assign a thread-safe, mutual exclusive lock of the receiver from the channel.
        //This ensure will be able to lock the receiver, keeping only one thread accessing data from this data at a time.
        //The lock as to be acquire then release for each thread to encourage atomicity on a receiver job.
        //This lets multiple worker owner to the receiver while the Mutex has only one worker gets a Job structure at a time.
        let mut workers = Vec::with_capacity(size); //Creates an mutable worker Vec that will be created according to the ThreadPool size parameter.

        for id in 0..size{ //Loop through the available threads.
            workers.push(Worker::new(id,Arc::clone(&receiver))); //Add worker instance for a thread from a cloned thread-safe reference to the receiver.
            //Each worker will need a reference to the receiver to get back the response, thread-safe Arc for multiple threads receiving the single response message asynchronously.
        }
        ThreadPool{ //Creates an instance of ThreadPool with workers and a sender wrapped in Some.
            workers, //Store JoinHandler to take closure of logic to run through and send it back to the existing thread.
            sender: Some(sender), //Sender data to write back to the stream.
        }
    }
    pub fn execute<F>(&self, f: F) //Execute function with a self reference to itself (the thread).
    where //Below executes based on the condition
    F: FnOnce() + Send + 'static, //Type alias that can be called only once with an unknown static lifetime scope the thread.
    //Send takes ownership and executes once, this trait bound transfers closure to threads with threads borrowing data to use.
    {
        let job = Box::new(f); //Smart pointer to allocates memory in the heap and places f type Job alias there.
        self.sender.as_ref().unwrap().send(job).unwrap(); //Sends the job through the channel.
    }
}

impl Drop for ThreadPool{ //Implementing a type of memory allocator for the ThreadPool to remove from memory.
    fn drop(&mut self){ //Take mutable reference if itself to implement drop.
        drop(self.sender.take()); //Take sender out of object to remove from memory, leave None after dropping.
    //This explicitly drops sender out of the pool before waiting for threads to finish.
    //This will address the main thread blocking indefinitely on a endless loop preventing deadlocks and starvation.

        for worker in &mut self.workers{ //Loop through workers from a mutable reference of the workers being dropped.
            println!("Shutting down worker {}", worker.id); //Print out the current worker id.

            if let Some(thread) = worker.thread.take(){ //If the thread has a worker, take the mutable reference and bind to a conditional statement.
                thread.join().unwrap(); //Wait for that thread to finish its operation before dropping out of scope
            }
        }
    }
}
struct Worker{ //Worker object that holds an ID and a thread
    id: usize, //Id to keep track of a worker performing with a thread.
    thread: Option<thread::JoinHandle<()>>, //The thread a worker is working on, used as optional to return Some or none.
}

impl Worker{ //implemented operations of the worker object
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker{ //Initialize a new worker with an id and receiver field that is thread safe mutual exclusive lock to ensure atomicity on thread work in this receiver.
        let thread = thread::spawn(move||loop{ //Spawn a thread, move it in the closure and force ownership of receiver with the environment variables.
           let message = receiver.lock().unwrap().recv(); //Lock and unwraps to receive a message, this a result of a job and possible error in receiving.
            //Since we force ownership, we lock to make sure no other threads are effecting the receiver object while the Worker's operates.

            match message{ //On condition of the message (the Job object)
                Ok(job) => { //If its a valid job and it is initiated then print to the console.
                    println!("Worker {id} got a job; executing."); //Print the id of the worker that is executing the job.
                    job();
                }
                Err(_) => { //On any error, print the disconnection message to the console. This say the worker is done and is being dropped.
                    println!("Worker {id} disconnected; shutting down."); //Print the id of the worker that is being dropped.
                    break; //Terminate the execution of the loop.
                }
            }

        });
        Worker{id, thread: Some(thread),} //Worker object with its ID field and thread the worker would perform the job in.
    }
}