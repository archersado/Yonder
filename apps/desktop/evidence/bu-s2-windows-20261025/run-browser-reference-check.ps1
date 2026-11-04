$base = Join-Path $env:ProgramFiles 'Citro Labs\ego lite\Application'
$ego = Get-ChildItem -LiteralPath $base -Filter 'ego-browser.exe' -Recurse -File |
    Sort-Object { $_.Directory.Name -as [version] } -Descending |
    Select-Object -First 1 -ExpandProperty FullName
if (-not $ego) {
    throw 'ego-browser.exe not found'
}
& $ego nodejs -e "await import('file:///F:/workspace/yonder/apps/desktop/evidence/bu-s2-windows-20261025/browser-reference-round-04.mjs')"
exit $LASTEXITCODE
