#![allow(non_snake_case)] //Tells the compiler to ignore snake case warning(project was not named in snake case)

use MultiThreadProject::ThreadPool; //This brings in the ThreadPool object into scope for main from the lib.rs file.

use std::{ //This brings in three sub-modules into scope along with its sub-modules/objects.
    fs, //File operations functions, used to read HTML scripts that will be used as responses.
    io::{prelude::*, BufReader}, //IO operations with two sub-modules. Used to write and read streams using in communicating through channels.
    net::{TcpListener,TcpStream}, //Net operations that add TCP listeners for incoming request and the stream that the requests/response are traveling through.
};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap(); //Create a listener instance that is bounded to this address and binds to the port while unwrapping any possible errors. Returns a Result<T,E>.
    let pool = ThreadPool::new(4); //Creates a thread pool holding 4 max threads. The max limit passed in the constructor parameter.

    for stream in listener.incoming().take(2){ //Take in 2 requests that will come from the address and shut down after the second thread completes its operation.
        //This request will be listening at this address/port and loop through the requests with a defined limit to shut down and dispose of the stream automatically at the end of take(n).
        let stream = stream.unwrap(); //Unwraps a result of TCP Stream and takes the stream or the error associated with it.

        pool.execute(||{ //Execute the thread's operations by taking the closure that is implement and define within the ThreadPool data structure.
            handle_connection(stream); //The closure will capture the stream as part of the environment, borrowing the stream to pass in the function to handle write operations.
         });
      }
    println!("Shutting down."); //The listener is finished looping through incoming streams, stream is shut down and dropped from scope while the listener is not longer taking in connections.
}

fn handle_connection(mut stream: TcpStream){ //Connection handler that will take a mutable reference to the TCP stream.
    let buf_reader = BufReader::new(&mut stream); //Creates a reader from a mutable reference of stream to read the incoming data in the stream.
    //This borrows the stream to be mutable, that is needed when writing to the stream for different get requests.
    let request_line = buf_reader.lines().next().unwrap().unwrap(); //Iterates over the lines, returns a string and unwraps errors at lines() and next(). If any errors occur to panic the application.


    let(status_line, filename) = if request_line == "GET / HTTP/1.1"{ //Create a HTTP GET request with its parameters.
        // Assign a tuple holding the status HTTP (message and code) along with the filename of the HTML script to send down the response stream.
        ("HTTP/1.1 200 OK", "hello.html") //Send status code and HTTP version along with the body contents.
    }else{
        ("HTTP/1.1 404 NOT FOUND","404.html") //Send a 404 status code and an error HTML body.
    };

    let contents = fs::read_to_string(filename).unwrap(); //Read a file to a string and unwraps errors to panic the application.
    let length = contents.len(); //Gets the length of the HTML body. This is a parameter in the HTTP GET request.

    let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}"); //Create the response string to send down the stream.
    //The HTTP requests consists of parameters: status line (status code, http version, and status message), content-length, and the string contents separated by new lines.

    stream.write_all(response.as_bytes()).unwrap(); //Write bytes to the stream to send back, unwrap errors.

}