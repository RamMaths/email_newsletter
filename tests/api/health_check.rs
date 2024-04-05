use super::helpers;

#[tokio::test]
async fn health_check_works() {
    // Arrange
    let app = helpers::spawn_app().await;
    let client = reqwest::Client::new();
    //We use reqwest to perform HTTP request against our application
    
    println!("{}", &app.address);
    
    //Act
    let response = client
        .get(format!("{}/health_check", &app.address))
        .send()
        .await
        .expect("Failed to execute request");

    //Assert
    assert!(response.status().is_success());
}
