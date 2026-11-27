@echo off
powershell.exe -NoProfile -ExecutionPolicy Bypass -Command "& '%~dp0scripts\play-rtx3060.ps1' %*"
if errorlevel 1 pause
