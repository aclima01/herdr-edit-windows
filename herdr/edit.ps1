# herdr-edit actions (Windows). Opens, closes, or toggles the editor split.
#
#   edit.ps1 toggle   open the editor, or close it if already open
#   edit.ps1 open     open the editor, no-op if one is open
#   edit.ps1 close    close every editor pane, no-op if none
#
# The workspace's editor is any pane labeled "edit" in the live pane list. There is no state file.
# Actions refuse loudly (exit 1, one stderr line) and report success on stdout. On Windows herdr's
# `bash` resolves to WSL, which can't see the Windows repo, so this is a native PowerShell port.
# PowerShell parses JSON natively, so unlike a unix script this needs no jq. Each step is tolerant:
# a transient herdr hiccup must never read as "no editor" and stack a duplicate on toggle.
[CmdletBinding()]
param([string]$Mode = 'toggle')

$ErrorActionPreference = 'SilentlyContinue'

$script:H = if ($env:HERDR_BIN_PATH) { $env:HERDR_BIN_PATH } else { 'herdr' }
$pluginId = if ($env:HERDR_PLUGIN_ID) { $env:HERDR_PLUGIN_ID } else { 'aclima.edit' }
$label    = 'edit'

function Herdr { & $script:H @args 2>$null }

function Refuse([string]$msg) {
    [Console]::Error.WriteLine("edit: $msg")
    exit 1
}

$ws   = $env:HERDR_WORKSPACE_ID
$pane = $env:HERDR_PANE_ID
$cwd  = ''

# Prefer the focused pane's cwd, else the workspace cwd, so the editor opens on the repo the user
# is looking at.
if ($env:HERDR_PLUGIN_CONTEXT_JSON) {
    try {
        $ctx = $env:HERDR_PLUGIN_CONTEXT_JSON | ConvertFrom-Json
        if ($ctx.focused_pane_cwd) { $cwd = $ctx.focused_pane_cwd }
        elseif ($ctx.workspace_cwd) { $cwd = $ctx.workspace_cwd }
    }
    catch {}
}

if (-not $ws) { Refuse 'no workspace context (invoke from inside herdr)' }

# One pane-list snapshot serves the whole run. A failed listing must not read as "no editor" —
# that would stack a duplicate on toggle and false-succeed a close.
$panesJson = (Herdr pane list --workspace $ws | Out-String).Trim()
if (-not $panesJson) { Refuse "herdr pane list failed for $ws" }
try {
    $panes = ($panesJson | ConvertFrom-Json).result.panes
}
catch {
    Refuse "herdr pane list failed for $ws"
}

$existing = @()
if ($panes) {
    $existing = @($panes | Where-Object { $_.label -eq $label } | ForEach-Object { $_.pane_id })
}

# Plain `pane close`, not `plugin pane close`: the plugin-pane registry does not survive a herdr
# restart and would strand the pane.
function Close-All {
    $closed = @()
    $failed = @()
    foreach ($p in $existing) {
        if (-not $p) { continue }
        Herdr pane close $p | Out-Null
        if ($LASTEXITCODE -eq 0) { $closed += $p } else { $failed += $p }
    }
    if ($failed.Count -gt 0) { Refuse "failed to close $($failed -join ' ') in $ws" }
    "closed $($closed -join ' ') in $ws"
}

if ($Mode -eq 'close') {
    if ($existing.Count -eq 0) { "close: nothing open in $ws"; exit 0 }
    Close-All
    exit 0
}
elseif ($Mode -eq 'toggle') {
    if ($existing.Count -gt 0) { Close-All; exit 0 }
}
elseif ($Mode -eq 'open') {
    if ($existing.Count -gt 0) { "open: already open ($($existing -join ' ')) in $ws"; exit 0 }
}
else {
    Refuse "unknown mode '$Mode' (toggle | open | close)"
}

# Opening from here. Attach the right split to the focused pane, else the workspace's first pane.
if (-not $pane -and $panes) { $pane = $panes[0].pane_id }
if (-not $pane) { Refuse "no pane to attach to in $ws" }

$openArgs = @('--placement', 'split', '--direction', 'right', '--target-pane', $pane)
if ($cwd) { $openArgs += @('--cwd', $cwd) }

$openJson = (Herdr plugin pane open --plugin $pluginId --entrypoint edit `
        @openArgs --no-focus | Out-String).Trim()
$new = ''
if ($openJson) {
    try { $new = ($openJson | ConvertFrom-Json).result.plugin_pane.pane.pane_id } catch {}
}
if (-not $new) { Refuse 'herdr plugin pane open failed' }
"opened $new (split) in $ws"
