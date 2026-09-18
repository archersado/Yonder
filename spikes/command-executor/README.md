# CM-S1 结构化命令执行 Spike

macOS：

```bash
rustc --edition 2024 macos-probe.rs -o /private/tmp/yonda-command-macos-probe
/private/tmp/yonda-command-macos-probe
```

测试Shell仅用于构造父子进程样本；产品候选接口不接受Shell字符串。Windows Job Object样本按用户决定暂缓。
