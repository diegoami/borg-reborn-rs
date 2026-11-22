@echo off
rem One headless Borg game (#823): play-headless.cmd OUTDIR [SEED [RACE [CLASS [TURNS]]]]
rem See play-headless.ps1, which this runs.
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0play-headless.ps1" %*
exit /b %ERRORLEVEL%
