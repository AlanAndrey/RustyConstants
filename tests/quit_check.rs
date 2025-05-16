use actix_web::{App, HttpServer, web, rt as actix_rt};
use rusty_constants::quit; // Assuming 'quit' is an HTTP handler you might use
use std::sync::mpsc;
use std::time::Duration;
use std::thread;
use std::net::TcpStream;
use rand::Rng; // Trait needed for gen_range

#[actix_web::test]
async fn test_server_shuts_down_when_signalled_via_channel() {
    // 1. Setup MPSC channel:
    // - tx_signal_to_server: Used by this test to send the shutdown command.
    // - rx_server_listens_for_signal: Used by the server's internal thread to receive the command.
    let (tx_signal_to_server, rx_server_listens_for_signal) = mpsc::channel::<()>();

    // This clone of the sender would be passed to the Actix application state.
    // If your `quit` HTTP handler uses this app_data to send a signal, it would trigger the same mechanism.
    let app_data_sender = web::Data::new(tx_signal_to_server.clone());

    // 2. Setup server address with a random port to avoid conflicts
    let mut rng = rand::rng(); // Correct way to get a thread-local RNG
    let port = rng.random_range(10000..=55535); // Generate a port in a common dynamic/private range
    let address = format!("127.0.0.1:{}", port);
    let address_for_server_binding = address.clone();
    let address_for_connection_test = address.clone();

    // 3. Start the Actix server in a separate thread
    let server_thread_handle = thread::spawn(move || {
        let system = actix_rt::System::new(); // Create an Actix runtime for the server thread
        system.block_on(async move {
            let server = HttpServer::new(move || {
                App::new()
                    .app_data(app_data_sender.clone()) // Provide the sender to the app
                    .service(quit) // Register the 'quit' service (even if not called via HTTP in this test)
            })
            .bind(&address_for_server_binding)
            .expect("Failed to bind server to address")
            .workers(1) // Keep it simple for testing
            .run();

            let server_handle = server.handle();

            // This inner thread is crucial: it listens for the shutdown signal
            // on `rx_server_listens_for_signal` and stops the server.
            thread::spawn(move || {
                println!("[Server Listener] Waiting for shutdown signal...");
                match rx_server_listens_for_signal.recv() {
                    Ok(()) => {
                        println!("[Server Listener] Shutdown signal received. Telling server to stop.");
                        server_handle.stop(true); // Gracefully stop the server
                    }
                    Err(e) => {
                        eprintln!("[Server Listener] Error receiving shutdown signal: {}. Server might not stop.", e);
                    }
                }
                println!("[Server Listener] Exiting.");
            });

            println!("[Server Main Task] Server started on {}. Waiting for shutdown...", address_for_server_binding);
            // `server.await` will block until the server has fully shut down.
            if let Err(e) = server.await {
                eprintln!("[Server Main Task] Server.await returned an error: {:?}", e);
            }
            println!("[Server Main Task] Server has shut down.");
        });
        println!("[Server Thread] Actix system finished.");
    });

    // 4. Wait briefly for the server to start up.
    println!("[Test] Waiting for server to start at {}...", address_for_connection_test);
    thread::sleep(Duration::from_secs(1)); // Adjust if server startup is slower

    // 5. Verify the server is running by attempting a TCP connection.
    println!("[Test] Verifying server is running...");
    if TcpStream::connect(&address_for_connection_test).is_err() {
        // If initial connection fails, server might still be starting. Wait a bit longer.
        thread::sleep(Duration::from_secs(2));
        assert!(
            TcpStream::connect(&address_for_connection_test).is_ok(),
            "Server failed to start or is not listening on {}.", address_for_connection_test
        );
    }
    println!("[Test] Server confirmed to be running.");

    // 6. Trigger the shutdown by sending a signal on `tx_signal_to_server`.
    // This simulates the event that should cause the server to stop.
    println!("[Test] Sending shutdown signal to the server...");
    tx_signal_to_server.send(()).expect("Test: Failed to send shutdown signal via MPSC channel");

    // 7. Wait for the server to process the shutdown signal and stop.
    println!("[Test] Waiting for server to shut down...");
    thread::sleep(Duration::from_secs(2)); // Allow time for graceful shutdown procedures

    // 8. Verify the server is no longer running. The connection attempt should fail.
    println!("[Test] Verifying server has shut down...");
    assert!(
        TcpStream::connect(&address_for_connection_test).is_err(),
        "Server is still running after shutdown signal was sent. It should have stopped."
    );
    println!("[Test] Server confirmed to be shut down.");

    // 9. Ensure the server thread has completed and catch any panics from it.
    println!("[Test] Waiting for server thread to join...");
    server_thread_handle.join().expect("Server thread panicked during execution");
    println!("[Test] Test completed successfully.");
}