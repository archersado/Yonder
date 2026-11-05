# 生态与许可关键主张独立核验

核验对象：生态/许可Round 1摘要；访问日期2026-11-04。

## 结果

- **验证**：Apache-2.0允许商业使用、修改与目标代码再分发并授予专利许可；需附许可证、标明修改、保留相关声明，商标不授予。来源：[项目LICENSE](https://github.com/vercel-labs/agent-desktop/blob/main/LICENSE)、[Apache License 2.0](https://www.apache.org/licenses/LICENSE-2.0.html)、[ASF FAQ](https://www.apache.org/foundation/license-faq)。
- **验证**：仓库创建于2026-02-19，48个Release；v1.0.0为10-28，v1.1.0为11-03，1.x约6天。来源：[仓库API](https://api.github.com/repos/vercel-labs/agent-desktop)、[Release API](https://api.github.com/repos/vercel-labs/agent-desktop/releases?per_page=100)、[CHANGELOG](https://github.com/vercel-labs/agent-desktop/blob/main/CHANGELOG.md)。
- **验证**：公开贡献计数约91.4%集中于lahfir（224/245人工计数）；只能证明公开提交集中，不能等同组织工时。来源：[contributors API](https://api.github.com/repos/vercel-labs/agent-desktop/contributors?per_page=100&anon=1)。
- **验证**：v1.1.0在干净Windows 11 ARM64缺`VCRUNTIME140.dll`时失败；修复PR #543在访问时仍Open。x64干净VM失败是合理推断，直接复现为ARM64。来源：[Issue #542](https://github.com/vercel-labs/agent-desktop/issues/542)、[PR #543](https://github.com/vercel-labs/agent-desktop/pull/543)。
- **验证并收窄**：Release矩阵有三平台x64/ARM64的6个CLI、6个FFI制品及checksums；主CI有多平台成功记录。原生macOS/Linux E2E workflow存在，但公开API未见独立持续成功运行证据，不能据“定义”写成“持续通过”。来源：[v1.1.0](https://github.com/vercel-labs/agent-desktop/releases/tag/v1.1.0)、[release.yml](https://raw.githubusercontent.com/vercel-labs/agent-desktop/main/.github/workflows/release.yml)、[ci.yml](https://raw.githubusercontent.com/vercel-labs/agent-desktop/main/.github/workflows/ci.yml)。
- **未验证的缺失主张**：公开资料未发现LTS/弃用周期，不能证明组织内部或未来不存在；[SUPPORT.md](https://github.com/vercel-labs/agent-desktop/blob/main/SUPPORT.md)明确best-effort且无SLA。
- **验证（公开发行物范围）**：v1.1.0未发布SPDX/CycloneDX等逐版本SBOM，Release资产与workflow只期待平台归档、checksums和源码包。
- **推翻笼统“无签名”**：Release workflow通过`actions/attest-build-provenance`为归档和checksums生成Sigstore provenance，GitHub attestation API可查；但核验者直接检查Windows PE的Security Directory为零，macOS Mach-O无`LC_CODE_SIGNATURE`。正确表述是“有供应链来源证明，无Windows Authenticode与Apple原生代码签名/公证”。来源：[release.yml](https://raw.githubusercontent.com/vercel-labs/agent-desktop/main/.github/workflows/release.yml)、[GitHub artifact attestations](https://docs.github.com/en/actions/concepts/security/artifact-attestations)、[v1.1.0](https://github.com/vercel-labs/agent-desktop/releases/tag/v1.1.0)。
