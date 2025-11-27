# run.ps1 - Start both the FastAPI backend and Tauri desktop app
# Usage: .\run.ps1

param(
    [switch]$Backend,
    [switch]$Frontend,
    [switch]$Help
)

if ($Help) {
    Write-Host @"
FLL Project - Development Server Launcher

Usage: .\run.ps1 [options]

Options:
  -Backend    Start only the backend server
  -Frontend   Start only the frontend (Tauri desktop app)
  -Help       Show this help message

By default, both servers are started.

Requirements:
  - Python 3.8+ with virtual environment activated
  - Rust and Cargo installed
  - Trunk (web bundler) installed

URLs:
  - Backend API: http://localhost:8000
  - API Docs:    http://localhost:8000/docs
  - Frontend:    Tauri desktop window (auto-opens)
"@
    exit 0
}

# Determine which servers to start
$startBackend = -not $Frontend -or $Backend
$startFrontend = -not $Backend -or $Frontend

$projectRoot = $PSScriptRoot
$backendDir = Join-Path $projectRoot "backend"
$frontendDir = Join-Path $projectRoot "frontend"

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "FLL Project - Development Server" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# Start Backend
if ($startBackend) {
    Write-Host "Starting FastAPI backend..." -ForegroundColor Green
    Write-Host "  Location: $backendDir" -ForegroundColor Gray
    Write-Host "  Command: uvicorn backend.main:app --reload" -ForegroundColor Gray
    
    # Activate venv and start backend
    $venvActivate = Join-Path $projectRoot "venv\Scripts\Activate.ps1"
    $backendCommand = "& '$venvActivate'; python -m uvicorn backend.main:app --reload --host 0.0.0.0 --port 8000"
    
    $backendProcess = Start-Process -PassThru -FilePath "powershell" `
        -ArgumentList "-NoExit", "-Command", $backendCommand `
        -WorkingDirectory $projectRoot `
        -NoNewWindow
    
    Write-Host "  Backend PID: $($backendProcess.Id)" -ForegroundColor Gray
    Start-Sleep -Seconds 3
}

# Start Frontend
if ($startFrontend) {
    $tauriDir = Join-Path $frontendDir "fll"
    Write-Host "Starting Tauri desktop application..." -ForegroundColor Green
    Write-Host "  Location: $tauriDir" -ForegroundColor Gray
    Write-Host "  Command: dx serve" -ForegroundColor Gray
    Write-Host "  (This may take a moment on first run...)" -ForegroundColor Gray
    
    $frontendProcess = Start-Process -PassThru -FilePath "cargo" `
        -ArgumentList "tauri", "dev" `
        -WorkingDirectory $tauriDir
    
    Write-Host "  Frontend PID: $($frontendProcess.Id)" -ForegroundColor Gray
    Start-Sleep -Seconds 5
}

Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "✓ Application is running!" -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

if ($startBackend) {
    Write-Host "Backend API:" -ForegroundColor Cyan
    Write-Host "  http://localhost:8000" -ForegroundColor Yellow
    Write-Host "  API Docs: http://localhost:8000/docs" -ForegroundColor Yellow
    Write-Host ""
}

if ($startFrontend) {
    Write-Host "Frontend (Tauri):" -ForegroundColor Cyan
    Write-Host "  Desktop window will open automatically" -ForegroundColor Yellow
    Write-Host ""
}

Write-Host "Press Ctrl+C to stop all servers" -ForegroundColor Gray
Write-Host ""

# Keep the script running until user presses Ctrl+C
try {
    while ($true) {
        Start-Sleep -Seconds 1
    }
}
catch {
    Write-Host "Interrupted by user" -ForegroundColor Yellow
}
finally {
    Write-Host ""
    Write-Host "Stopping servers..." -ForegroundColor Yellow
    
    if ($startBackend -and $backendProcess) {
        Write-Host "  Stopping backend (PID: $($backendProcess.Id))..." -ForegroundColor Gray
        Stop-Process -Id $backendProcess.Id -Force -ErrorAction SilentlyContinue
        # Also kill child processes
        Get-Process | Where-Object { $_.Parent.Id -eq $backendProcess.Id } | Stop-Process -Force -ErrorAction SilentlyContinue
    }
    
    if ($startFrontend -and $frontendProcess) {
        Write-Host "  Stopping frontend (PID: $($frontendProcess.Id))..." -ForegroundColor Gray
        Stop-Process -Id $frontendProcess.Id -Force -ErrorAction SilentlyContinue
        # Also kill child processes (cargo and its children)
        Get-Process | Where-Object { $_.Parent.Id -eq $frontendProcess.Id } | Stop-Process -Force -ErrorAction SilentlyContinue
    }
    
    Write-Host "Done!" -ForegroundColor Green
Write-Host "Done!" -ForegroundColor Green
}
