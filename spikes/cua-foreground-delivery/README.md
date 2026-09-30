# trycua单次前台投递Spike

只验证固定`@trycua/cua-driver@0.30.4`候选，不修改产品依赖。

```bash
npm ci
swiftc -framework AppKit ../cua-driver-comparison/input-fixture-macos.swift -o fixture
node schema-probe.mjs evidence/schema-0.30.4.json
swiftc -framework AppKit decoy-macos.swift -o decoy
node foreground-probe.mjs ./fixture ./decoy evidence/candidate-0.30.4.json
sleep 1
YONDER_CUA_VERSION=0.25.0 YONDER_CUA_SDK="$(pwd)/../../apps/desktop/cua/node_modules/@trycua/cua-driver/dist/index.js" node foreground-probe.mjs ./fixture ./decoy evidence/baseline-0.25.0.json
sleep 1
YONDER_DROP_DECOY=1 YONDER_CUA_VERSION=0.25.0 YONDER_CUA_SDK="$(pwd)/../../apps/desktop/cua/node_modules/@trycua/cua-driver/dist/index.js" node foreground-probe.mjs ./fixture ./decoy evidence/recovery-failure-0.25.0.json || test $? -eq 1
python3 verify.py
```

输出只含Schema字段布尔与稳定枚举；不得提交截图、窗口标题或完整SDK Payload。
