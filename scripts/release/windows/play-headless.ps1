# One headless Borg game from an unpacked Windows release package (#823):
# CI's check that the package plays the reference games. Not shipped (owner,
# v0.1.0-alpha.1); the package folder is $env:BORG_PACKAGE_DIR.
#   $env:BORG_PACKAGE_DIR = DIR; scripts\release\windows\play-headless.ps1 OUTDIR [SEED [RACE [CLASS [TURNS]]]] Plays a new character with
# no window (the borgh frontend) until game turn TURNS (default 50000;
# 0 = no limit) or its death, and writes OUTDIR\keys.log, messages.log,
# state.log and summary.json. RACE and CLASS are indices into
# lib\gamedata\p_race.txt and class.txt (defaults: seed 101, Human Warrior).
# The settings are borg\borg.txt with the race and class set, as the source
# tree's harness/run.sh sets them, so seed 101 0 0 20000 plays the same keys
# as its smoke-warrior-1 reference.
param(
    [Parameter(Mandatory = $true)][string]$OutDir,
    [UInt32]$Seed = 101, [UInt32]$Race = 0, [UInt32]$Class = 0, [UInt32]$Turns = 50000
)
$ErrorActionPreference = 'Stop'
if (-not $env:BORG_PACKAGE_DIR) { throw 'play-headless: set BORG_PACKAGE_DIR to the unpacked package' }
$pkg = (Resolve-Path $env:BORG_PACKAGE_DIR).Path
if (Test-Path $OutDir) { throw "play-headless: $OutDir exists; use a new directory" }
$out = (New-Item -ItemType Directory -Path $OutDir).FullName
foreach ($d in 'user', 'save', 'archive', 'panic', 'scores') { New-Item -ItemType Directory -Path (Join-Path $out $d) | Out-Null }
Copy-Item (Join-Path $pkg 'borg\*.txt') (Join-Path $out 'user')
# The same bytes as the Unix runner writes: UTF-8 without a BOM, LF kept.
$text = [IO.File]::ReadAllText((Join-Path $pkg 'borg\borg.txt'))
$text = [Regex]::Replace($text, '(?m)^borg_respawn_race = .*$', "borg_respawn_race = $Race")
$text = [Regex]::Replace($text, '(?m)^borg_respawn_class = .*$', "borg_respawn_class = $Class")
$text = [Regex]::Replace($text, '(?m)^borg_cheat_death = .*$', 'borg_cheat_death = FALSE')
[IO.File]::WriteAllText((Join-Path $out 'user\borg.txt'), $text, (New-Object Text.UTF8Encoding $false))
# The engine's own paths use '/', which Windows accepts.
$o = $out -replace '\\', '/'
$gameArgs = @('-mborgh', '-n', '-uBorg', "-duser=$o/user", "-dsave=$o/save", "-darchive=$o/archive",
    "-dpanic=$o/panic", "-dscores=$o/scores", '--', '-seed', "$Seed", '-race', "$Race", '-class', "$Class",
    '-turns', "$Turns", '-menu-letters', '-keylog', "$o/keys.log", '-msglog', "$o/messages.log",
    '-statelog', "$o/state.log", '-summary', "$o/summary.json", '-screen')
# One command line, each argument with a space quoted.
$line = ($gameArgs | ForEach-Object { if ($_ -match ' ') { '"' + $_ + '"' } else { $_ } }) -join ' '
$psi = New-Object Diagnostics.ProcessStartInfo
$psi.FileName = Join-Path $pkg 'angband.exe'
$psi.Arguments = $line
$psi.WorkingDirectory = $pkg   # the game finds .\lib relative to the working directory
$psi.UseShellExecute = $false
$psi.RedirectStandardError = $true
$p = [Diagnostics.Process]::Start($psi)
$err = $p.StandardError.ReadToEnd()
$p.WaitForExit()
[IO.File]::WriteAllText((Join-Path $out 'stderr.log'), $err)
if ($p.ExitCode -ne 0 -or -not (Test-Path (Join-Path $out 'summary.json'))) {
    throw "play-headless: the game failed (exit $($p.ExitCode)); see $out\stderr.log"
}
Get-Content (Join-Path $out 'summary.json')
