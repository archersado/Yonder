# 独立 Verification Goal：Agent 会话临时附件 Spike

日期：2026-09-20
关联Story：CX-S2
状态：等待非实现者复核

## 验证范围

复核隔离Spike是否在不修改产品协议、Gateway或持久化的前提下，验证同一会话内有界分块、完整性、跨会话隔离及失败清理。不得把本结果解释为确认卡已能向Agent发送截图。

## 验证矩阵

| 项目 | 验证方法 | 通过条件 | 当前结果 |
|---|---|---|---|
| 成功边界 | 运行1字节、跨分块、4 MiB样本 | 全部完成引用消费，最终缓冲为0 | 待复核 |
| 帧与总量 | 读取结构化结果 | 最大帧≤65,536字节，总量≤4 MiB | 待复核 |
| 非法输入 | 超限、乱序、哈希错误 | 全部稳定拒绝并清理所属附件 | 待复核 |
| 会话隔离 | 第二会话引用第一会话附件 | 拒绝且不读取、不删除原会话附件 | 待复核 |
| 生命周期 | deadline到期、所属会话断连 | 缓冲归零 | 待复核 |
| 数据边界 | 检查证据和仓库变更 | 不含附件正文、base64或哈希；不改产品协议/运行时 | 待复核 |

## 实现者预检

- `python3 spikes/agent-input-attachment/probe.py`：退出0；4 MiB样本分为88块，最大帧64,258字节，最终缓冲0。
- `openspec validate cx-s2-agent-attachment-spike --strict`：通过。
- `PYTHONPATH=scripts python3 -m unittest scripts/test_check_architecture.py`：16项通过。
- `PATH=/Users/archersado/.cargo/bin:$PATH python3 scripts/check_architecture.py`：通过。
- 结构化证据：`spikes/agent-input-attachment/evidence/result.json`。

## 独立结论

待非实现者填写。通过后才能把AD-CX-02从Proposed更新为Accepted，并另建产品协议Change。
