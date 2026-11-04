# VI-S1 Windows稀疏MSIX身份构建验证（2026-11-02）

结论：**PASS（只构建子范围）；签名、信任、注册和语音对照尚未执行。**

## 已验证

- 使用Windows SDK 10.0.22621.0的`MakeAppx.exe`成功生成只含身份清单的外部位置MSIX。
- 同一`windows-explicit-probe.exe`内嵌side-by-side `msix`元数据，构建脚本提取后确认`packageName`、`publisher`、`applicationId`与包清单逐项一致。
- 清单使用`AllowExternalContent=true`、`win32App`、`mediumIL`与`runFullTrust`；没有创建第二产品入口或复制用户数据。
- 安装前`--identity-only`返回`package_identity_present=false`和`opened_microphone=false`，符合未注册基线。
- 当前用户没有同Subject的个人证书或TrustedPeople证书，包注册数为0。构建阶段没有创建证书、修改信任、注册包或打开麦克风。

结构化证据：`spikes/voice-input/evidence/windows-20261102/msix-identity-build.json`。

## 剩余门禁

用户已确认安装状态变更，但当前用户签名信任被`0x800B0109`拒绝；官方未签名OID路线也被本机构建分别以`0x80073D2C/0x80080204`拒绝。所有尝试均自动回滚，包、证书、状态文件和进程为零。详见`spikes/voice-input/evidence/windows-20261102/msix-identity-registration-attempts.json`。下一步需要用户另行明确授权管理员/UAC，把短期测试证书公钥放入`LocalMachine\TrustedPeople`；不得使用Root存储。注册后还需先以`--identity-only`证明运行时身份为true，再另行授权真实语音样本，验证结束后精确卸载包与证书。
