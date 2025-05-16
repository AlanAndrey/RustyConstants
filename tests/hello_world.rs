use actix_web::{test, App};
use rusty_constants::{hello};

#[actix_web::test]
async fn test_hello_integration() {
    // Arrange: Set up the test app
    /// Represents the application instance configured for testing.
    ///
    /// This instance is initialized with a new `App` and has the `hello` service registered.
    /// It is prepared for use in integration tests to simulate client requests and verify responses.
    let app = test::init_service(
        App::new()
            .service(hello)
    ).await;

    // Act: Create a test request
    let req = test::TestRequest::get().uri("/").to_request();
    let resp = test::call_service(&app, req).await;

    // Assert: Verify the response
    assert!(resp.status().is_success());

    // Check the response body
    let body = test::read_body(resp).await;
    assert_eq!(body, "Hello, world!");
}