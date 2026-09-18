use std::{collections::HashMap, ffi::c_void, ptr::NonNull};
use yonder_application::{ExecutionAttempt, computer_use::WorkTarget, work_focus::{FocusFailure, FocusOutcome, WorkFocusPort, WorkRef}};

unsafe extern "C" {
    fn yonda_work_ref_capture(pid:i32,window_id:u32,output:*mut *mut c_void,sec:*mut u64,usec:*mut u64)->i32;
    fn yonda_work_ref_focus(reference:*mut c_void)->i32;
    fn yonda_work_ref_release(reference:*mut c_void);
}

struct NativeRef(NonNull<c_void>);
// AXUIElement引用由CF retain持有，访问始终受TaskHost互斥锁串行化。
unsafe impl Send for NativeRef {}
impl Drop for NativeRef { fn drop(&mut self) { unsafe { yonda_work_ref_release(self.0.as_ptr()) } } }

pub struct MacWorkFocus { references: HashMap<(String,String),(WorkRef,NativeRef)> }
impl MacWorkFocus { pub fn new()->Self { Self { references:HashMap::new() } } }
impl Default for MacWorkFocus { fn default()->Self { Self::new() } }

fn failure(code:i32)->FocusFailure { match code { 1=>FocusFailure::PermissionUnavailable,2=>FocusFailure::ProcessChanged,3=>FocusFailure::WindowMissing,4=>FocusFailure::MappingNotUnique,5=>FocusFailure::ActivationFailed,6=>FocusFailure::VerificationFailed,7=>FocusFailure::GeometryChanged,_=>FocusFailure::ReferenceUnavailable } }

impl WorkFocusPort for MacWorkFocus {
    fn capture(&mut self,attempt:&ExecutionAttempt,target:&WorkTarget)->Result<WorkRef,FocusFailure> {
        let id=format!("work_{}",attempt.accepted_sequence);
        let key=(attempt.task_id.clone(),id.clone());
        if let Some((existing,_))=self.references.get(&key) {
            if existing.task_id==attempt.task_id&&existing.attempt_id==attempt.attempt_id&&existing.pid==target.pid&&existing.window_id==target.window_id {return Ok(existing.clone())}
            return Err(FocusFailure::ReferenceUnavailable);
        }
        let mut raw=std::ptr::null_mut();let mut sec=0;let mut usec=0;
        let code=unsafe { yonda_work_ref_capture(i32::try_from(target.pid).map_err(|_| FocusFailure::ReferenceUnavailable)?,target.window_id,&mut raw,&mut sec,&mut usec) };
        if code!=0 {return Err(failure(code))} let native=NativeRef(NonNull::new(raw).ok_or(FocusFailure::ReferenceUnavailable)?);
        let reference=WorkRef { work_ref_id:id.clone(),task_id:attempt.task_id.clone(),step_id:attempt.step_id.clone(),attempt_id:attempt.attempt_id.clone(),worker_instance_id:attempt.worker_instance_id.clone(),host_session_id:attempt.host_session_id.clone(),pid:target.pid,window_id:target.window_id,process_start_seconds:sec,process_start_microseconds:usec };
        self.references.insert(key,(reference.clone(),native));Ok(reference)
    }
    fn focus(&mut self,reference:&WorkRef)->FocusOutcome {
        let key=(reference.task_id.clone(),reference.work_ref_id.clone());
        let Some((stored,native))=self.references.get(&key) else{return FocusOutcome::Refused(FocusFailure::ReferenceUnavailable)};
        if stored!=reference{return FocusOutcome::Refused(FocusFailure::ReferenceUnavailable)}
        match unsafe { yonda_work_ref_focus(native.0.as_ptr()) } {0=>FocusOutcome::Focused,code=>FocusOutcome::Refused(failure(code))}
    }
    fn release(&mut self,reference:&WorkRef) {let key=(reference.task_id.clone(),reference.work_ref_id.clone());if self.references.get(&key).is_some_and(|(stored,_)|stored==reference){self.references.remove(&key);}}
}
