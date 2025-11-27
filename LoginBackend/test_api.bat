@echo off
REM Test script for Login Backend API

echo.
echo ====================================
echo   Testing Login Backend API
echo ====================================
echo.

REM Colors and formatting
setlocal enabledelayedexpansion

REM Check if curl is available
where curl >nul 2>&1
if %errorlevel% neq 0 (
    echo Error: curl is not installed or not in PATH
    echo Please install curl from https://curl.se/download.html
    pause
    exit /b 1
)

echo Waiting for servers to start (5 seconds)...
timeout /t 5 /nobreak

echo.
echo Test 1: Rust API Health Check
echo URL: http://localhost:8000/health
echo.
curl -s http://localhost:8000/health | findstr /R ".*" >nul
if %errorlevel% neq 0 (
    echo FAILED: Could not connect to Rust API on port 8000
    echo Make sure to start the API first with: cargo run
    pause
    exit /b 1
)
curl -s http://localhost:8000/health
echo.
echo.

echo Test 2: Rust API - Search User (Valid Credentials)
echo URL: http://localhost:8000/search_user
echo Request: {"username":"admin","password":"admin123"}
echo.
curl -s -X POST http://localhost:8000/search_user ^
  -H "Content-Type: application/json" ^
  -d "{\"username\":\"admin\",\"password\":\"admin123\"}"
echo.
echo.

echo Test 3: Rust API - Search User (Invalid Credentials)
echo URL: http://localhost:8000/search_user
echo Request: {"username":"admin","password":"wrongpassword"}
echo.
curl -s -X POST http://localhost:8000/search_user ^
  -H "Content-Type: application/json" ^
  -d "{\"username\":\"admin\",\"password\":\"wrongpassword\"}"
echo.
echo.

echo Test 4: Rails API - Health Check
echo URL: http://localhost:3000/up
echo.
curl -s http://localhost:3000/up | findstr /R ".*" >nul
if %errorlevel% neq 0 (
    echo FAILED: Could not connect to Rails API on port 3000
    echo Rails may still be starting...
)
curl -s http://localhost:3000/up
echo.
echo.

echo Test 5: Rails API - Login (Valid Credentials)
echo URL: http://localhost:3000/api/v1/users/login
echo Request: {"username":"admin","password":"admin123"}
echo.
curl -s -X POST http://localhost:3000/api/v1/users/login ^
  -H "Content-Type: application/json" ^
  -d "{\"username\":\"admin\",\"password\":\"admin123\"}"
echo.
echo.

echo Test 6: Rails API - Login (Invalid Credentials)
echo URL: http://localhost:3000/api/v1/users/login
echo Request: {"username":"admin","password":"wrongpassword"}
echo.
curl -s -X POST http://localhost:3000/api/v1/users/login ^
  -H "Content-Type: application/json" ^
  -d "{\"username\":\"admin\",\"password\":\"wrongpassword\"}"
echo.
echo.

echo ====================================
echo   All Tests Complete
echo ====================================
echo.
echo If all tests above returned valid JSON responses,
echo your API is working correctly!
echo.

pause
