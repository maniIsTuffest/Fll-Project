@echo off
REM Login Backend Startup Script for Windows
REM This script starts both the Rust backend and Rails API

echo.
echo ========================================
echo    Login Backend - Starting System
echo ========================================
echo.

REM Check if we're in the right directory
if not exist "main\Cargo.toml" (
    echo Error: Could not find main\Cargo.toml
    echo Please run this script from the LoginBackend directory
    pause
    exit /b 1
)

if not exist "api\api\Gemfile" (
    echo Error: Could not find api\api\Gemfile
    echo Please run this script from the LoginBackend directory
    pause
    exit /b 1
)

echo Checking prerequisites...
echo.

REM Check for Rust
rustc --version >nul 2>&1
if %errorlevel% neq 0 (
    echo Error: Rust is not installed or not in PATH
    echo Please install Rust from https://rustup.rs/
    pause
    exit /b 1
)
echo [OK] Rust found

REM Check for Ruby
ruby --version >nul 2>&1
if %errorlevel% neq 0 (
    echo Error: Ruby is not installed or not in PATH
    echo Please install Ruby from https://www.ruby-lang.org/en/downloads/
    pause
    exit /b 1
)
echo [OK] Ruby found

REM Check for Bundler
bundle --version >nul 2>&1
if %errorlevel% neq 0 (
    echo Error: Bundler is not installed
    echo Please run: gem install bundler
    pause
    exit /b 1
)
echo [OK] Bundler found

echo.
echo ========================================
echo    Installing Dependencies
echo ========================================
echo.

REM Install Rails dependencies
echo Installing Rails dependencies...
cd api\api
call bundle install
if %errorlevel% neq 0 (
    echo Error: Failed to install Rails dependencies
    cd ..\..
    pause
    exit /b 1
)
cd ..\..
echo [OK] Rails dependencies installed

echo.
echo ========================================
echo    Starting Services
echo ========================================
echo.
echo Rust API Server:  http://localhost:8000
echo Rails API:        http://localhost:3000
echo.
echo Press Ctrl+C to stop all services
echo.

REM Start the Rust backend (which will start Rails as a subprocess)
cd main
cargo run

cd ..
pause
