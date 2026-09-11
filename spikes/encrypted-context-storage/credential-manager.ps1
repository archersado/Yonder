$ErrorActionPreference = 'Stop'
$source = @'
using System;
using System.Runtime.InteropServices;
using System.Security.Cryptography;

public static class CredentialProbe {
  const uint Generic = 1;
  const uint LocalMachine = 2;

  [StructLayout(LayoutKind.Sequential, CharSet = CharSet.Unicode)]
  struct Credential {
    public uint Flags, Type;
    public string TargetName, Comment;
    public System.Runtime.InteropServices.ComTypes.FILETIME LastWritten;
    public uint CredentialBlobSize;
    public IntPtr CredentialBlob;
    public uint Persist, AttributeCount;
    public IntPtr Attributes;
    public string TargetAlias, UserName;
  }

  [DllImport("advapi32.dll", CharSet = CharSet.Unicode, SetLastError = true)]
  static extern bool CredWriteW(ref Credential credential, uint flags);
  [DllImport("advapi32.dll", CharSet = CharSet.Unicode, SetLastError = true)]
  static extern bool CredReadW(string target, uint type, uint flags, out IntPtr credential);
  [DllImport("advapi32.dll", CharSet = CharSet.Unicode, SetLastError = true)]
  static extern bool CredDeleteW(string target, uint type, uint flags);
  [DllImport("advapi32.dll")] static extern void CredFree(IntPtr credential);

  public static string Run() {
    string target = "Yonder/E0-S6/" + Guid.NewGuid().ToString("N");
    byte[] secret = new byte[32];
    using (var random = RandomNumberGenerator.Create()) random.GetBytes(secret);
    IntPtr blob = Marshal.AllocHGlobal(secret.Length);
    bool written = false, roundtrip = false, deleted = false, absent = false;
    try {
      Marshal.Copy(secret, 0, blob, secret.Length);
      var value = new Credential { Type = Generic, TargetName = target, CredentialBlobSize = (uint)secret.Length, CredentialBlob = blob, Persist = LocalMachine, UserName = "Yonder" };
      written = CredWriteW(ref value, 0);
      if (!written) throw new System.ComponentModel.Win32Exception(Marshal.GetLastWin32Error());
      IntPtr stored;
      if (!CredReadW(target, Generic, 0, out stored)) throw new System.ComponentModel.Win32Exception(Marshal.GetLastWin32Error());
      try {
        var read = (Credential)Marshal.PtrToStructure(stored, typeof(Credential));
        byte[] copy = new byte[read.CredentialBlobSize];
        Marshal.Copy(read.CredentialBlob, copy, 0, copy.Length);
        roundtrip = copy.Length == secret.Length;
        for (int i = 0; roundtrip && i < copy.Length; i++) roundtrip &= copy[i] == secret[i];
        Array.Clear(copy, 0, copy.Length);
      } finally { CredFree(stored); }
      deleted = CredDeleteW(target, Generic, 0);
      IntPtr missing;
      absent = !CredReadW(target, Generic, 0, out missing);
      if (!absent) CredFree(missing);
      return "{\"written\":" + written.ToString().ToLowerInvariant() + ",\"roundtrip\":" + roundtrip.ToString().ToLowerInvariant() + ",\"deleted\":" + deleted.ToString().ToLowerInvariant() + ",\"absent_after_delete\":" + absent.ToString().ToLowerInvariant() + "}";
    } finally {
      if (written && !deleted) CredDeleteW(target, Generic, 0);
      for (int i = 0; i < secret.Length; i++) Marshal.WriteByte(blob, i, 0);
      Marshal.FreeHGlobal(blob);
      Array.Clear(secret, 0, secret.Length);
    }
  }
}
'@
Add-Type -TypeDefinition $source
[CredentialProbe]::Run()
