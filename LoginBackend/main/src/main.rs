mod db;

use rocket::{serde::json::Json, get, post, routes, launch, Build, Rocket};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
struct LoginRequest {
    username: String,
    password: String,
}

#[derive(Serialize, Debug)]
struct LoginResponse {
    success: bool,
    message: String,
    data: Option<db::UserResponse>,
}

#[derive(Serialize, Debug)]
struct ApiResponse<T: Serialize> {
    success: bool,
    message: String,
    data: Option<T>,
}

#[derive(Serialize, Debug)]
struct HealthResponse {
    status: String,
    api: String,
    version: String,
}

// Root path handler
#[get("/")]
fn index() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "running".to_string(),
        api: "Login Backend API".to_string(),
        version: "1.0.0".to_string(),
    })
}

// Health check endpoint
#[get("/health")]
fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        api: "Login Backend API".to_string(),
        version: "1.0.0".to_string(),
    })
}

// Login endpoint - main user-facing endpoint
#[post("/login", format = "json", data = "<request>")]
fn login(request: Json<LoginRequest>) -> Json<LoginResponse> {
    let username = &request.username;
    let password = &request.password;

    match db::search_user(username, password) {
        Ok(Some(user)) => {
            Json(LoginResponse {
                success: true,
                message: "Login successful".to_string(),
                data: Some(user.into()),
            })
        }
        Ok(None) => {
            Json(LoginResponse {
                success: false,
                message: "Invalid username or password".to_string(),
                data: None,
            })
        }
        Err(e) => {
            eprintln!("Database error: {}", e);
            Json(LoginResponse {
                success: false,
                message: format!("Database error: {}", e),
                data: None,
            })
        }
    }
}

// Search user endpoint - alias for login
#[post("/search_user", format = "json", data = "<request>")]
fn search_user(request: Json<LoginRequest>) -> Json<LoginResponse> {
    let username = &request.username;
    let password = &request.password;

    match db::search_user(username, password) {
        Ok(Some(user)) => {
            Json(LoginResponse {
                success: true,
                message: "User found".to_string(),
                data: Some(user.into()),
            })
        }
        Ok(None) => {
            Json(LoginResponse {
                success: false,
                message: "Invalid credentials".to_string(),
                data: None,
            })
        }
        Err(e) => {
            eprintln!("Database error: {}", e);
            Json(LoginResponse {
                success: false,
                message: format!("Database error: {}", e),
                data: None,
            })
        }
    }
}

// Get user info endpoint
#[post("/user/info", format = "json", data = "<request>")]
fn user_info(request: Json<LoginRequest>) -> Json<LoginResponse> {
    let username = &request.username;
    let password = &request.password;

    match db::search_user(username, password) {
        Ok(Some(user)) => {
            Json(LoginResponse {
                success: true,
                message: "User information retrieved".to_string(),
                data: Some(user.into()),
            })
        }
        Ok(None) => {
            Json(LoginResponse {
                success: false,
                message: "User not found".to_string(),
                data: None,
            })
        }
        Err(e) => {
            eprintln!("Database error: {}", e);
            Json(LoginResponse {
                success: false,
                message: format!("Database error: {}", e),
                data: None,
            })
        }
    }
}

#[launch]
fn rocket() -> Rocket<Build> {
    println!("\n========================================");
    println!("   Login Backend API - Starting");
    println!("========================================\n");

    // Initialize the database
    if let Err(e) = db::init_db() {
        eprintln!("Failed to initialize database: {}", e);
        eprintln!("Attempting to continue...");
    } else {
        println!("✓ Database initialized successfully");
    }

    println!("\n========================================");
    println!("   API Endpoints Available");
    println!("========================================");
    println!("\nHealth & Info:");
    println!("  GET  http://localhost:9000/");
    println!("  GET  http://localhost:9000/health");
    println!("\nAuthentication:");
    println!("  POST http://localhost:9000/login");
    println!("  POST http://localhost:9000/search_user");
    println!("\nUser Info:");
    println!("  POST http://localhost:9000/user/info");
    println!("\n========================================");
    println!("   API Running on http://localhost:9000");
    println!("========================================\n");

    rocket::build()
        .configure(rocket::Config::figment().merge(("port", 9000)))
        .mount("/", routes![
            index,
            health,
            login,
            search_user,
            user_info,
        ])
}
