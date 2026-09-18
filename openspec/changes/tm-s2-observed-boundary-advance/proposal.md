# Proposal：Observed 连续步骤边界

Story：TM-S2。

为CUA/BUA多步执行补齐共同边界：已Observe的attempt可普通结束，任务保持running且保留任务级资源Permit；随后接受下一步骤并准备新attempt。prepared、unknown、pending控制和旧身份不得推进。
