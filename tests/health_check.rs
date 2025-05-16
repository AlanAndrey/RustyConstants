use actix_web::{test, App};
use rusty_constants::{health_check};

#[actix_web::test]
async fn test_hello_integration() {
    // Arrange: Set up the test app
    /// Initializes the application with the `health_check` service for testing purposes.
    /// The `app` variable holds an instance of the test service, ready to receive requests.
    let app = test::init_service(
        App::new()
            .service(health_check)
    ).await;

    // Act: Create a test request
    let req = test::TestRequest::get().uri("/health").to_request();
    let resp = test::call_service(&app, req).await;

    // Assert: Verify the response
    assert!(resp.status().is_success());

    // Check the response body
    let body = test::read_body(resp).await;
    assert_eq!(body, "Tutto Bene!");
}