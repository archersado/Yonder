param(
    [Parameter(Mandatory = $true)]
    [string]$DocumentPath,
    [Parameter(Mandatory = $true)]
    [string]$WordPath
)

$ErrorActionPreference = 'Stop'

Add-Type -TypeDefinition @'
using System;
using System.IO;
using System.Runtime.InteropServices;
using Microsoft.Win32.SafeHandles;

public static class WindowsFileIdentity
{
    [StructLayout(LayoutKind.Sequential)]
    private struct ByHandleFileInformation
    {
        public uint FileAttributes;
        public System.Runtime.InteropServices.ComTypes.FILETIME CreationTime;
        public System.Runtime.InteropServices.ComTypes.FILETIME LastAccessTime;
        public System.Runtime.InteropServices.ComTypes.FILETIME LastWriteTime;
        public uint VolumeSerialNumber;
        public uint FileSizeHigh;
        public uint FileSizeLow;
        public uint NumberOfLinks;
        public uint FileIndexHigh;
        public uint FileIndexLow;
    }

    [DllImport("kernel32.dll", SetLastError = true)]
    private static extern bool GetFileInformationByHandle(
        SafeFileHandle handle,
        out ByHandleFileInformation information);

    public static string Read(string path)
    {
        using (var stream = new FileStream(path, FileMode.Open, FileAccess.Read,
            FileShare.ReadWrite | FileShare.Delete))
        {
            ByHandleFileInformation information;
            if (!GetFileInformationByHandle(stream.SafeFileHandle, out information))
                throw new System.ComponentModel.Win32Exception(Marshal.GetLastWin32Error());
            ulong index = ((ulong)information.FileIndexHigh << 32) | information.FileIndexLow;
            return information.VolumeSerialNumber.ToString("X8") + ":" + index.ToString("X16");
        }
    }
}
'@

$root = Join-Path ([IO.Path]::GetTempPath()) ("yonder-fi-s1-" + [Guid]::NewGuid().ToString('N'))
$word = $null
try {
    [IO.Directory]::CreateDirectory($root) | Out-Null
    $realDirectory = Join-Path $root 'real'
    [IO.Directory]::CreateDirectory($realDirectory) | Out-Null
    $source = Join-Path $realDirectory 'source.docx'
    Copy-Item -LiteralPath $DocumentPath -Destination $source
    $hardlink = Join-Path $root 'hardlink.docx'
    $symlink = Join-Path $root 'symlink.docx'
    $aliasKind = 'file-symbolic-link'
    New-Item -ItemType HardLink -Path $hardlink -Target $source | Out-Null
    $symlinkCreated = $true
    $symlinkError = $null
    try {
        New-Item -ItemType SymbolicLink -Path $symlink -Target $source | Out-Null
    } catch {
        $symlinkCreated = $false
        $symlinkError = $_.Exception.Message
        $aliasDirectory = Join-Path $root 'directory-junction'
        try {
            New-Item -ItemType Junction -Path $aliasDirectory -Target $realDirectory | Out-Null
            $symlink = Join-Path $aliasDirectory 'source.docx'
            $symlinkCreated = $true
            $aliasKind = 'directory-junction'
        } catch {
            $symlinkError = "$symlinkError; junction: $($_.Exception.Message)"
        }
    }

    $sourceIdentity = [WindowsFileIdentity]::Read($source)
    $hardlinkSame = $sourceIdentity -eq [WindowsFileIdentity]::Read($hardlink)
    $symlinkSame = $symlinkCreated -and $sourceIdentity -eq [WindowsFileIdentity]::Read($symlink)

    $first = [IO.File]::Open($source, 'Open', 'ReadWrite', 'None')
    try {
        $secondLockRejected = $false
        try {
            $second = [IO.File]::Open($hardlink, 'Open', 'ReadWrite', 'None')
            $second.Dispose()
        } catch [IO.IOException] {
            $secondLockRejected = $true
        }
    } finally {
        $first.Dispose()
    }

    $rootFull = [IO.Path]::GetFullPath($root).TrimEnd('\') + '\'
    $escaped = [IO.Path]::GetFullPath((Join-Path $root '..\escape.docx'))
    $escapeRejected = -not $escaped.StartsWith($rootFull, [StringComparison]::OrdinalIgnoreCase)

    $temporaryOutput = Join-Path $root '.output.tmp'
    $finalOutput = Join-Path $root 'output.docx'
    [IO.File]::WriteAllText($temporaryOutput, 'verified')
    [IO.File]::Move($temporaryOutput, $finalOutput)
    $atomicCommit = (Test-Path -LiteralPath $finalOutput) -and -not (Test-Path -LiteralPath $temporaryOutput)

    $word = Start-Process -FilePath $WordPath -ArgumentList @('/n', $source) -PassThru
    $deadline = (Get-Date).AddSeconds(20)
    do {
        Start-Sleep -Milliseconds 500
        $word.Refresh()
    } while (-not $word.HasExited -and $word.MainWindowHandle -eq 0 -and (Get-Date) -lt $deadline)
    if ($word.HasExited -or $word.MainWindowHandle -eq 0) { throw 'Word未创建可验证窗口' }
    Start-Sleep -Seconds 2
    $officeLockRejected = $false
    try {
        $exclusive = [IO.File]::Open($source, 'Open', 'ReadWrite', 'None')
        $exclusive.Dispose()
    } catch [IO.IOException] {
        $officeLockRejected = $true
    }

    [ordered]@{
        platform = 'windows'
        identity_kind = 'volume-serial-file-index'
        hardlink_same = $hardlinkSame
        symlink_created = $symlinkCreated
        symlink_same = $symlinkSame
        alias_kind = $aliasKind
        symlink_error = $symlinkError
        second_lock_rejected = $secondLockRejected
        escape_rejected = $escapeRejected
        atomic_commit = $atomicCommit
        office = 'Microsoft Word'
        office_window_created = $true
        office_lock_rejected = $officeLockRejected
        passed = $hardlinkSame -and $symlinkSame -and $secondLockRejected -and
            $escapeRejected -and $atomicCommit -and $officeLockRejected
    } | ConvertTo-Json -Compress
} finally {
    if ($null -ne $word -and -not $word.HasExited) {
        Stop-Process -Id $word.Id -Force -ErrorAction SilentlyContinue
        Wait-Process -Id $word.Id -Timeout 10 -ErrorAction SilentlyContinue
    }
    if (Test-Path -LiteralPath $root) {
        foreach ($attempt in 1..10) {
            try {
                Remove-Item -LiteralPath $root -Recurse -Force -ErrorAction Stop
                break
            } catch {
                if ($attempt -eq 10) { throw }
                Start-Sleep -Milliseconds 500
            }
        }
    }
}
