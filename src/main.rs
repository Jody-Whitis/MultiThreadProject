#![allow(non_snake_case)] //tell the compiler to ignore snake case warning(project was not named in snake case)

use MultiThreadProject::ThreadPool; //bring in ThreadPool object into scope for main.

use std::{ //bring in three sub-modules into scope along with its sub-modules/objects.
    fs, //file operations, used to read html scripts that will be used as responses.
    io::{prelude::*, BufReader}, //io operations with two sub-modules. Used to write and read streams using in communicating through channels.
    net::{TcpListener,TcpStream}, //net operations that add TCP listeners for incoming request and the stream that the requests/response are traveling through.
};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap(); //create a listener instance binding at this address/port while unwrapping any possible errors. Returns a Result<T,E>.
    let pool = ThreadPool::new(4); //create a thread pool holding 4 max threads.

    for stream in listener.incoming().take(2){ //take in 2 request that will come from the address and shut down after the second thread completes its operation.
        //This request will be listening at this address/port and loop through the requests with a defined limit to shut down and dispose of the stream automatically.
        let stream = stream.unwrap(); //unwraps a Result of TCP Stream and takes the stream or the error associated with it.

        pool.execute(||{ //execute the thread's operations by taking the closure this is implement and define within the ThreadPool data structure.
            handle_connection(stream); //pass in a mutable stream to the function.
        });
      }
    println!("Shutting down."); //listener is finished looping incoming streams, stream is shut down and dropped from scope while the listener is not longer taking in connections.
}

fn handle_connection(mut stream: TcpStream){ //connection handler that will take a mutable reference to the tcp stream.
    let buf_reader = BufReader::new(&mut stream); //create a reader from a mutable pointer of stream to read the incoming data in the stream.
    let request_line = buf_reader.lines().next().unwrap().unwrap(); //iterates over the lines and returns a string and unwraps errors at lines() and next() if any occur to panic the application.


    let(status_line, filename) = if request_line == "GET / HTTP/1.1"{ //create a http get request with its parameters.
        // Assign a tuple holding the status HTTP along with the filename of the html script to send down the response stream.
        ("HTTP/1.1 200 OK", "hello.html") //send status code and http version along with the body contents.
    }else{
        ("HTTP/1.1 404 NOT FOUND","404.html") //send a 404 status code and an error html body.
    };

    let contents = fs::read_to_string(filename).unwrap(); //read a file to a string and unwraps errors to panic the application.
    let length = contents.len(); //get the length of the html body. This is a parameter in the HTTP get request.

    let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}"); //create the response string to send down the stream.
    //The http requests consists of parameters: status line (status code, http version, and status message, content-length, and the string contents separated by newlines.

    stream.write_all(response.as_bytes()).unwrap(); //write bytes to the stream to send back, unwrap errors.

}