//! 任务状态纯计算；数据库事务提交后才可发布返回的迁移。

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Status {
    Created,
    Running,
    WaitingForUser,
    Paused,
    Interrupted,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Action {
    Start,
    Pause,
    WaitForUser,
    Resume,
    Interrupt,
    Complete,
    Fail,
    Cancel,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransitionError {
    InvalidTransition,
    SequenceExhausted,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Transition {
    pub previous: Status,
    pub next: Status,
    pub sequence: u64,
}

impl Status {
    /// sequence 是持久化的当前序号；调用方须在同事务中检查并提交。
    pub fn transition(self, sequence: u64, action: Action) -> Result<Transition, TransitionError> {
        use Action::*;
        let next = match (self, action) {
            (Self::Created, Start) => Self::Running,
            (Self::Running, Pause) => Self::Paused,
            (Self::Running, WaitForUser) => Self::WaitingForUser,
            (Self::Paused | Self::WaitingForUser | Self::Interrupted, Resume) => Self::Running,
            (Self::Running, Interrupt) => Self::Interrupted,
            (Self::Running, Complete) => Self::Completed,
            (Self::Running, Fail) => Self::Failed,
            (Self::Created | Self::Running | Self::Paused | Self::WaitingForUser | Self::Interrupted, Cancel) => Self::Cancelled,
            _ => return Err(TransitionError::InvalidTransition),
        };
        Ok(Transition {
            previous: self,
            next,
            sequence: sequence.checked_add(1).ok_or(TransitionError::SequenceExhausted)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn task_lifecycle_requires_explicit_resume_and_protects_terminal_states() {
        let started = Status::Created.transition(1, Action::Start).unwrap();
        let interrupted = started.next.transition(started.sequence, Action::Interrupt).unwrap();
        assert_eq!(interrupted.next, Status::Interrupted);
        assert_eq!(interrupted.next.transition(3, Action::Start), Err(TransitionError::InvalidTransition));
        let resumed = interrupted.next.transition(interrupted.sequence, Action::Resume).unwrap();
        let completed = resumed.next.transition(resumed.sequence, Action::Complete).unwrap();
        assert_eq!(completed.sequence, 5);
        for terminal in [Status::Completed, Status::Failed, Status::Cancelled] {
            for action in [Action::Start, Action::Resume, Action::Cancel, Action::Complete] {
                assert_eq!(terminal.transition(5, action), Err(TransitionError::InvalidTransition));
            }
        }
        assert_eq!(Status::Running.transition(u64::MAX, Action::Complete), Err(TransitionError::SequenceExhausted));
        assert_eq!(Status::Paused.transition(2, Action::Complete), Err(TransitionError::InvalidTransition));
    }
}
