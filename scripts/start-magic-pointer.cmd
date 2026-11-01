@echo off
setlocal
set "BINARY=%~dp0magic-pointer.exe"

if not exist "%BINARY%" set "BINARY=%~dp0..\magic-pointer.exe"

if not exist "%BINARY%" (
  echo magic-pointer.exe is missing.
  echo Download the Windows x64 artifact from GitHub Actions and place it beside this script.
  exit /b 1
)

"%BINARY%" %*
exit /b %ERRORLEVEL%
