@echo off
rem Watch the Borg play in this console window (#823): watch-borg.cmd [NAME]
rem The curses frontend, in a console of at least 80x24 (Windows Terminal
rem works). Saves and settings live in data\ next to this file; the first
rem start copies the Borg's settings (borg\) there, later starts keep any
rem change made to data\user\borg.txt. Create or load a character, then
rem press Ctrl-Z and z to start the Borg; any key stops it.
setlocal
set "PKG=%~dp0"
set "NAME=%~1"
if "%NAME%"=="" set "NAME=RustBorg"
if "%BORG_PLAY_DATA_DIR%"=="" (set "DATA=%PKG%data") else (set "DATA=%BORG_PLAY_DATA_DIR%")
for %%d in (user save archive panic scores) do if not exist "%DATA%\%%d" mkdir "%DATA%\%%d"
for %%f in ("%PKG%borg\*.txt") do if not exist "%DATA%\user\%%~nxf" copy /y "%%f" "%DATA%\user\" >nul
echo Create or load a character; Ctrl-Z then z starts the Borg. Any key stops it.
pushd "%PKG%"
angband.exe -mgcu "-duser=%DATA%\user" "-dsave=%DATA%\save" "-darchive=%DATA%\archive" "-dpanic=%DATA%\panic" "-dscores=%DATA%\scores" "-u%NAME%"
set RC=%ERRORLEVEL%
popd
exit /b %RC%
