//! 仅供 Desktop 组合根使用的 Preview 临时会话；不写入任务或存储。

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Phase { Idle, Selecting, Capturing, Reviewing, Submitting }

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error { Busy, InvalidPhase, ImageTooLarge }

pub struct Session { phase: Phase, image: Option<String> }

impl Default for Session { fn default() -> Self { Self { phase: Phase::Idle, image: None } } }

impl Session {
    pub fn begin(&mut self) -> Result<(), Error> {
        if self.phase != Phase::Idle { return Err(Error::Busy); }
        self.image = None; self.phase = Phase::Selecting; Ok(())
    }
    pub fn capture(&mut self) -> Result<(), Error> {
        if self.phase != Phase::Selecting { return Err(Error::InvalidPhase); }
        self.phase = Phase::Capturing; Ok(())
    }
    /// Base64 仅在本进程的临时会话内保留，限制为约 16 MiB PNG。
    pub fn review(&mut self, image: String) -> Result<String, Error> {
        if self.phase != Phase::Capturing { return Err(Error::InvalidPhase); }
        if image.is_empty() || image.len() > 22_400_000 { self.clear(); return Err(Error::ImageTooLarge); }
        self.image = Some(image); self.phase = Phase::Reviewing;
        Ok(self.image.as_ref().expect("刚写入的预览").clone())
    }
    pub fn review_without_image(&mut self) -> Result<(), Error> {
        if !matches!(self.phase, Phase::Selecting | Phase::Capturing) { return Err(Error::InvalidPhase); }
        self.image = None; self.phase = Phase::Reviewing; Ok(())
    }
    pub fn begin_submission(&mut self) -> Result<Option<String>, Error> {
        if self.phase != Phase::Reviewing { return Err(Error::InvalidPhase); }
        self.phase = Phase::Submitting;
        Ok(self.image.clone())
    }
    pub fn clear(&mut self) { self.image = None; self.phase = Phase::Idle; }
    pub fn snapshot(&self) -> (Phase, usize) { (self.phase, self.image.as_ref().map_or(0, String::len)) }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn every_terminal_path_drops_the_image() {
        let mut session = Session::default();
        session.begin().unwrap(); session.capture().unwrap(); session.review("image".into()).unwrap();
        assert_eq!(session.snapshot(), (Phase::Reviewing, 5));
        assert_eq!(session.begin_submission(), Ok(Some("image".into())));
        assert_eq!(session.snapshot(), (Phase::Submitting, 5));
        session.clear(); assert_eq!(session.snapshot(), (Phase::Idle, 0));
        assert!(matches!(session.begin(), Ok(()))); assert!(matches!(session.begin(), Err(Error::Busy)));
    }

    #[test]
    fn rejected_capture_also_returns_to_idle() {
        let mut session = Session::default();
        session.begin().unwrap(); session.capture().unwrap();
        assert_eq!(session.review("x".repeat(22_400_001)), Err(Error::ImageTooLarge));
        assert_eq!(session.snapshot(), (Phase::Idle, 0));
    }

    #[test]
    fn review_and_submit_can_omit_an_image() {
        let mut session = Session::default();
        session.begin().unwrap(); session.review_without_image().unwrap();
        assert_eq!(session.snapshot(), (Phase::Reviewing, 0));
        assert_eq!(session.begin_submission(), Ok(None));
        session.clear(); assert_eq!(session.snapshot(), (Phase::Idle, 0));
    }
}
