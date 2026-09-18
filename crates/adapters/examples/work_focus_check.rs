#[cfg(target_os="macos")]
fn main() {
    use std::io::{self,BufRead};
    use yonder_adapters::work_focus::MacWorkFocus;
    use yonder_application::{AttemptPhase,ExecutionAttempt,computer_use::WorkTarget,work_focus::{FocusOutcome,WorkFocusPort}};
    let args=std::env::args().collect::<Vec<_>>();
    let target=WorkTarget { pid:args.get(1).and_then(|v|v.parse().ok()).unwrap_or(0),window_id:args.get(2).and_then(|v|v.parse().ok()).unwrap_or(0) };
    let attempt=ExecutionAttempt { task_id:"native-focus".into(),step_id:"focus".into(),attempt_id:"attempt".into(),worker_instance_id:"worker".into(),host_session_id:"host".into(),phase:AttemptPhase::Observed,accepted_sequence:1 };
    let mut port=MacWorkFocus::new();
    let reference=match port.capture(&attempt,&target) {Ok(value)=>value,Err(error)=>{println!("{}",serde_json::json!({"ready":false,"error":format!("{error:?}")}));return}};
    println!("{}",serde_json::json!({"ready":true,"work_ref_id":reference.work_ref_id,"start":[reference.process_start_seconds,reference.process_start_microseconds]}));
    for line in io::stdin().lock().lines().map_while(Result::ok) {
        match line.as_str() {
            "focus"=>println!("{}",serde_json::json!({"outcome":match port.focus(&reference){FocusOutcome::Focused=>"focused".into(),FocusOutcome::Refused(reason)=>format!("{reason:?}")}})),
            "release"=>{port.release(&reference);println!("{}",serde_json::json!({"released":true}));},
            "quit"=>break,
            _=>println!("{}",serde_json::json!({"error":"invalid_command"})),
        }
    }
}

#[cfg(not(target_os="macos"))]
fn main() {}
