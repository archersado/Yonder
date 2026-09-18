# 设计

协议1.7新增`task.step.advance`，1.8新增`browser.execute`。Gateway完成认证、版本与所属任务检查后调用Application Browser Supervisor。首次动作取得Browser单并发，后续步骤复用占用；finish提交completed后释放。CLI仅映射Rust协议类型。
