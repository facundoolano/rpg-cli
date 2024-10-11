# Shell integration for Windows Terminal
#
# Note:
# - I couldn't figure out how to override `cd`, so I used `cdir` instead.
# - To avoid hitting the absolute path length limit, I made the created directories single-digit numbers.
# - It seems that `&&` can be used in PowerShell 7, but it's not available in version 5, so I omitted the error handling.
#
# See:
# - [about_Profiles - PowerShell | Microsoft Learn](https://learn.microsoft.com/en-us/powershell/module/microsoft.powershell.core/about/about_profiles?view=powershell-5.1)
# - [Migrating from Windows PowerShell 5.1 to PowerShell 7 - PowerShell | Microsoft Learn](https://learn.microsoft.com/en-us/powershell/scripting/whats-new/migrating-from-windows-powershell-51-to-powershell-7?view=powershell-7.4#separate-profiles)
#
Set-Variable -Option Constant -Name RPG -Value "C:\your\path\to\rpg-cli.exe"

function rpg() {
    & $RPG $args
    sync_rpg
}

function cdir() {
    & $RPG cd $args
    sync_rpg
}

function dn() {
    $current = (Get-Item $PWD).BaseName
    if ($current -match "^[0-9]+$") {
        $next = (([int]$current) + 1) % 10
        New-Item -ItemType Directory -ErrorAction SilentlyContinue $next > $null
        cdir $next
    } elseif (Test-Path "1") {
        cdir 1
    } else {
        New-Item -ItemType Directory -ErrorAction SilentlyContinue "dungeon\1" > $null
        cdir "dungeon/1"
    }
    rpg ls
}

function sync_rpg() {
    $pwd = & $RPG pwd
    Set-Location -Path $pwd
}
