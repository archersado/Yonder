# 许可证、生态与供应链研究摘要（Round 1）

研究者结论：不宜直接成为商业产品唯一且不可替换的CUA基础依赖；适合作为高优先级候选，用固定版本、可替换Port/Adapter、内部制品镜像和可维护fork做限量验证。

## Findings

1. `{claim: "项目为Apache-2.0，允许商业使用、修改和二进制再分发并含专利许可；需附许可证、保留声明、标明修改，商标权不授予。仓库未发现NOTICE。", source: "https://github.com/vercel-labs/agent-desktop/blob/main/LICENSE", publisher: "Vercel Labs", pub_date: "未标注", accessed: "2026-11-04", confidence: "高", class: "许可"}`
2. `{claim: "Cargo与npm均声明Apache-2.0；cargo-deny许可允许列表含Apache/MIT/MPL-2.0等并拒绝未知registry/git。发行前仍需从Cargo.lock生成SBOM与第三方声明。", source: "https://github.com/vercel-labs/agent-desktop/blob/main/Cargo.toml ; npm/package.json ; deny.toml", publisher: "Vercel Labs", pub_date: "main至2026-11-03", accessed: "2026-11-04", confidence: "高", class: "许可供应链"}`
3. `{claim: "关键Windows/macOS/Linux绑定多为Apache-2.0；core-foundation-rs与zbus的GitHub License API为NOASSERTION，虽cargo-deny接受，法律清单仍需人工确认具体版本文本。", source: "GitHub License API: uiautomation-rs/windows-rs/accessibility/x11rb/core-foundation-rs/zbus", publisher: "GitHub及上游维护者", pub_date: "动态", accessed: "2026-11-04", confidence: "中高", class: "关键依赖许可"}`
4. `{claim: "仓库创建于2026-02-19；访问时约13952 stars、954 forks、58 open issues、96 open PR，最近一天活动密集，热度高但积压显著。", source: "https://api.github.com/repos/vercel-labs/agent-desktop ; GitHub search/issues/commits API", publisher: "GitHub", pub_date: "2026-11-04", accessed: "2026-11-04", confidence: "高", class: "生态成熟度"}`
5. `{claim: "contributors API显示6名非Bot贡献者，lahfir 224次，其他人工约20次，约92%可见人工贡献集中一人；Vercel Labs归属不能替代多人维护或SLA。", source: "https://api.github.com/repos/vercel-labs/agent-desktop/contributors?per_page=100&anon=1", publisher: "GitHub", pub_date: "动态", accessed: "2026-11-04", confidence: "高", class: "关键人风险"}`
6. `{claim: "约8.5个月内已有48个Release；v1.0.0为2026-10-28，v1.1.0为11-03，1.x历史仅一周；0.9.0仍有响应envelope breaking change。", source: "https://github.com/vercel-labs/agent-desktop/releases ; CHANGELOG.md", publisher: "Vercel Labs", pub_date: "最新2026-11-03", accessed: "2026-11-04", confidence: "高", class: "版本稳定性"}`
7. `{claim: "核心是Rust CLI/C库，不内嵌模型；通过AX/UIA/AT-SPI操作accessibility tree，更像原生桌面自动化工具而非截图视觉CUA，无法覆盖无可访问性树界面。", source: "README.md ; AGENTS.md", publisher: "Vercel Labs", pub_date: "至2026-11-03", accessed: "2026-11-04", confidence: "高", class: "定位"}`
8. `{claim: "README标记Vercel Labs Product；代码/测试/发行超过demo，但Labs定位、短1.x历史和无SLA使其仍是快速演进工具而非稳定基础设施。", source: "README.md", publisher: "Vercel Labs", pub_date: "至2026-11-03", accessed: "2026-11-04", confidence: "高", class: "成熟度"}`
9. `{claim: "安装渠道有npm、GitHub Releases与macOS/Linux Homebrew；npm需Node>=18但Node只是launcher，原生Release可免Node；未发现浏览器/Python产品运行时依赖。", source: "docs/install.md ; npm/package.json", publisher: "Vercel Labs", pub_date: "至2026-11-03", accessed: "2026-11-04", confidence: "高", class: "运行依赖"}`
10. `{claim: "v1.1.0压缩CLI约Windows1.57–1.70MB、macOS2.10–2.32MB、Linux3.31–3.43MB；FFI约1.46–3.41MB，适合嵌入分发。", source: "https://api.github.com/repos/vercel-labs/agent-desktop/releases/latest", publisher: "GitHub/Vercel Labs", pub_date: "2026-11-03", accessed: "2026-11-04", confidence: "高", class: "分发体积"}`
11. `{claim: "v1.1.0 Windows在缺VCRUNTIME140.dll时静默退出，静态CRT修复PR尚未合入，说明干净终端smoke test有缺口。", source: "https://github.com/vercel-labs/agent-desktop/issues/542 ; https://github.com/vercel-labs/agent-desktop/pull/543", publisher: "项目社区", pub_date: "2026-11-04", accessed: "2026-11-04", confidence: "高", class: "发布阻断"}`
12. `{claim: "CI覆盖格式、MSRV、Clippy、架构、core/platform/CLI/FFI测试、三平台多架构、macOS原生E2E、Linux live E2E、Windows capability probe、CodeQL、Dependabot与供应链；最新main关键流程成功。", source: "GitHub Actions/workflows/API", publisher: "Vercel Labs/GitHub Actions", pub_date: "最近2026-11-03", accessed: "2026-11-04", confidence: "高", class: "CI测试"}`
13. `{claim: "发布流水线构建三平台x64/ARM64 CLI与FFI共12工件，发布SHA-256；Actions依赖多固定完整commit SHA，归档与source SHA校验较强。", source: "release.yml ; supply-chain.yml ; v1.1.0 release", publisher: "Vercel Labs", pub_date: "2026-11-03", accessed: "2026-11-04", confidence: "高", class: "供应链发布"}`
14. `{claim: "deny.toml暂忽略3个zbus 5传递依赖unmaintained advisory，因为Linux依赖zbus 5.19 blocking API，形成升级锁定债务。", source: "deny.toml ; crates/linux/Cargo.toml", publisher: "Vercel Labs", pub_date: "至2026-11-03", accessed: "2026-11-04", confidence: "高", class: "供应链风险"}`
15. `{claim: "Apache-2.0允许内部fork，JSON envelope/MCP/C ABI可作隔离边界；但替代三套原生accessibility/input/screenshot/helper的功能退出成本中高，不应让领域协议依赖其60个命令。", source: "README.md ; LICENSE ; CHANGELOG.md", publisher: "Vercel Labs", pub_date: "至2026-11-03", accessed: "2026-11-04", confidence: "中高", class: "锁定退出"}`

## Leads

- 在干净Windows x64/ARM64 VM验证CRT、签名、杀软；生成Cargo.lock SBOM并人工核对NOASSERTION/MPL依赖。
- 固定tag与SHA，对JSON/MCP/C ABI建立契约测试；目标应用覆盖Electron、Office/WPS、自绘UI、安全窗口和高DPI。
- 向维护者确认LTS、漏洞响应、签名/公证及商业支持。
- 封装最小动作Port并保留第二实现或内部fork，不传播其命令模型到领域层。

## Looked for but not found

- 未发现LTS/弃用周期/兼容承诺/SLA、覆盖率门禁、Windows Authenticode、macOS notarization、逐版本SBOM/NOTICE、生产规模/SLO或官方Scoop包。
