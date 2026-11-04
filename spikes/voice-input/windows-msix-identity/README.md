# Windows稀疏MSIX身份对照

本目录只构建VI-S1 Windows语音候选的稀疏身份包，不注册包、不创建或信任证书，也不打开麦克风：

```powershell
powershell -ExecutionPolicy Bypass -File .\build-unsigned.ps1
```

输出位于`%LOCALAPPDATA%\Yonder\Spikes\VoiceMsixIdentity\0.1.0.0`。`AppxManifest.xml`与`probe.manifest`中的`Name/Publisher/Application Id`必须保持一致；构建脚本会提取EXE内嵌清单并检查这三个值。

安装前后可使用只读模式确认运行时身份；该模式不访问语音API或麦克风：

```powershell
& "$env:LOCALAPPDATA\Yonder\Spikes\VoiceMsixIdentity\0.1.0.0\windows-explicit-probe.exe" --identity-only
```

签名、当前用户`TrustedPeople`证书信任、`Add-AppxPackage -ExternalLocation`注册和测试后卸载都属于安装状态变更，必须在用户单独明确确认后由后续脚本执行。PFX、私钥和密码不得写入仓库或验证证据。

取得用户确认后，Windows 11未签名测试身份使用专用OID和`-AllowUnsigned`注册纯身份稀疏包，不创建证书、不修改Root或Developer Mode，最后执行零采集身份检查：

```powershell
powershell -ExecutionPolicy Bypass -File .\install-unsigned-test-identity.ps1
```

验证结束后，清理脚本只按注册状态文件中的精确Package Full Name卸载本测试身份：

```powershell
powershell -ExecutionPolicy Bypass -File .\cleanup-test-identity.ps1
```

本机AppxDeployment已拒绝当前用户TrustedPeople和未签名OID路线。若用户另行授权管理员/UAC，先运行`build-machine-trusted.ps1`生成Publisher为`CN=Yonder Voice Spike Dev`的隔离构建，再在管理员PowerShell中运行`install-machine-trusted-test-identity.ps1`；它只写`LocalMachine\TrustedPeople`，不写Root。验证结束后在管理员PowerShell运行`cleanup-machine-trusted-test-identity.ps1`，按精确包全名和证书指纹清理。
