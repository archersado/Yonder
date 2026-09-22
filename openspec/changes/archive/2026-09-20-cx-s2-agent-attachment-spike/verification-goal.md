# 独立 Verification Goal：Agent 会话临时附件 Spike

日期：2026-09-20
关联Story：CX-S2
状态：PASS

## 验证范围

复核隔离Spike是否在不修改产品协议、Gateway或持久化的前提下，验证同一会话内有界分块、完整性、跨会话隔离及失败清理。不得把本结果解释为确认卡已能向Agent发送截图。

## 验证矩阵

| 项目 | 验证方法 | 通过条件 | 当前结果 |
|---|---|---|---|
| 成功边界 | 运行1字节、跨分块、4 MiB样本 | 全部完成引用消费，最终缓冲为0 | PASS |
| 帧与总量 | 读取结构化结果 | 最大帧≤65,536字节，总量≤4 MiB | PASS（64,258字节） |
| 非法输入 | 第二个/重复开始、超限、乱序、哈希错误 | 全部稳定拒绝并清理所属会话 | PASS |
| 会话隔离 | 第二会话引用第一会话附件 | 拒绝且不读取、不删除原会话附件 | PASS |
| 生命周期 | 输入拒绝/unknown、deadline到期、所属会话断连 | 缓冲归零 | PASS |
| 数据边界 | 检查证据和仓库变更 | 不含附件正文、base64或哈希；不改产品协议/运行时 | PASS |

## 实现者预检

- `python3 spikes/agent-input-attachment/probe.py`：退出0；4 MiB样本分为88块，最大帧64,258字节，最终缓冲0。
- `openspec validate cx-s2-agent-attachment-spike --strict`：通过。
- `PYTHONPATH=scripts python3 -m unittest scripts/test_check_architecture.py`：16项通过。
- `PATH=/Users/archersado/.cargo/bin:$PATH python3 scripts/check_architecture.py`：通过。
- 结构化证据：`spikes/agent-input-attachment/evidence/result.json`。

## 独立结论

首轮复核FAIL：发现同会话第二个或重复`begin`可并存/覆盖，且输入拒绝与unknown未单独取证。提交`b3d5ddb`修复后，非实现者在HEAD `b3d5ddb0e8d5db2e8e07afdebc483ce5abac7675`重新运行六行矩阵，全部PASS；证据SHA-256为`c014794803b1a72fd7a5cecae4d730d55c0c26f9c36f09f5f52cc9e9d05eb040`。本结论只允许接受AD-CX-02并建立产品Change，不代表产品附件或确认卡发送已经完成。
