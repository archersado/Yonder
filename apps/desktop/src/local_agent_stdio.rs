//! 正式组合根的私有stdio研发连接，不提供生产认证。
use std::{io::{self, BufRead, Read, Write}, sync::{Arc, Mutex}, time::{SystemTime, UNIX_EPOCH}};
use yonder_application::{AuthContext, gateway::{GatewaySession, Platform}};
use crate::TaskHost;

pub fn serve(host: Arc<Mutex<Option<TaskHost>>>) -> Result<(), Box<dyn std::error::Error>> {
    // 身份由可信测试组合根绑定；不接受请求内容选择身份。
    let mut session = GatewaySession::new(AuthContext::Agent("local-test-agent"), Platform::Macos);
    let mut input = io::stdin().lock();
    let mut output = io::stdout().lock();
    loop {
        let mut frame = Vec::new();
        // 必须在解析前限长；完整 JSON + 换行最多 64KiB。
        let read = input.by_ref().take(64 * 1024 + 1).read_until(b'\n', &mut frame)?;
        if read == 0 { break; }
        if read > 64 * 1024 || frame.last() != Some(&b'\n') { return Err("无效或过大测试帧".into()); }
        frame.pop();
        let now = u64::try_from(SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis())?;
        let response = host.lock().map_err(|_| "测试宿主锁不可用")?.as_mut().ok_or("测试宿主不可用")?.query_session(&mut session, &frame, now).map_err(|_| "Gateway 测试失败")?;
        output.write_all(&response)?;
        output.write_all(b"\n")?;
        output.flush()?;
    }
    Ok(())
}
