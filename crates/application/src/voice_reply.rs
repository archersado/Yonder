const MAX_REPLY_CHARS: usize = 64 * 1024;
const MAX_SEGMENT_CHARS: usize = 512;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SpeakRequest {
    pub generation: u64,
    pub sequence: u64,
    pub text: String,
}

pub trait VoiceSynthesisPort: Send + Sync {
    fn available(&self) -> bool;
    fn speak(&self, request: &SpeakRequest) -> Result<(), &'static str>;
    fn stop(&self);
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VoiceReplyError {
    Disabled,
    ReplyMismatch,
    ReplyTooLarge,
}

#[derive(Default)]
pub struct VoiceReplySession {
    enabled: bool,
    generation: u64,
    active_reply_id: Option<String>,
    pending: String,
    next_sequence: u64,
    total_chars: usize,
    in_code_block: bool,
    backtick_run: u8,
    reply_closed: bool,
    native_idle: bool,
}

impl VoiceReplySession {
    pub fn new(enabled: bool) -> Self {
        Self { enabled, ..Self::default() }
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }

    pub fn set_enabled(&mut self, enabled: bool) -> bool {
        if self.enabled == enabled {
            return false;
        }
        self.enabled = enabled;
        if !enabled {
            self.invalidate();
            return true;
        }
        false
    }

    pub fn begin(&mut self, reply_id: &str) -> Result<u64, VoiceReplyError> {
        if !self.enabled {
            return Err(VoiceReplyError::Disabled);
        }
        self.invalidate();
        self.active_reply_id = Some(reply_id.to_owned());
        Ok(self.generation)
    }

    pub fn push_delta(
        &mut self,
        reply_id: &str,
        delta: &str,
    ) -> Result<Vec<SpeakRequest>, VoiceReplyError> {
        self.ensure_reply(reply_id)?;
        let delta_chars = delta.chars().count();
        if self.total_chars.saturating_add(delta_chars) > MAX_REPLY_CHARS {
            return Err(VoiceReplyError::ReplyTooLarge);
        }
        self.total_chars += delta_chars;
        let mut requests = Vec::new();
        for character in delta.chars() {
            if character == '`' {
                self.backtick_run += 1;
                if self.backtick_run == 3 {
                    self.in_code_block = !self.in_code_block;
                    if !self.in_code_block {
                        self.pending.push_str("代码内容已省略。");
                        self.emit_ready(&mut requests, true);
                    }
                    self.backtick_run = 0;
                }
                continue;
            }
            self.backtick_run = 0;
            if self.in_code_block {
                continue;
            }
            self.pending.push(character);
            let boundary = matches!(character, '。' | '！' | '？' | '!' | '?' | '；' | ';' | '\n');
            self.emit_ready(&mut requests, boundary);
        }
        Ok(requests)
    }

    pub fn finish(&mut self, reply_id: &str) -> Result<Vec<SpeakRequest>, VoiceReplyError> {
        self.ensure_reply(reply_id)?;
        if self.in_code_block {
            self.pending.push_str("代码内容已省略。");
            self.in_code_block = false;
        }
        self.backtick_run = 0;
        self.reply_closed = true;
        let mut requests = Vec::new();
        self.emit_ready(&mut requests, true);
        Ok(requests)
    }

    pub fn preview(&mut self, text: &str) -> Result<Vec<SpeakRequest>, VoiceReplyError> {
        const PREVIEW_REPLY_ID: &str = "voice-preview";
        self.begin(PREVIEW_REPLY_ID)?;
        let mut requests = self.push_delta(PREVIEW_REPLY_ID, text)?;
        requests.extend(self.finish(PREVIEW_REPLY_ID)?);
        Ok(requests)
    }

    pub fn stop(&mut self) -> u64 {
        self.invalidate();
        self.generation
    }

    pub fn accepts_callback(&self, generation: u64) -> bool {
        self.enabled && self.active_reply_id.is_some() && self.generation == generation
    }

    pub fn generation(&self) -> u64 {
        self.generation
    }

    pub fn native_idle(&mut self, generation: u64) -> bool {
        if !self.accepts_callback(generation) {
            return false;
        }
        self.native_idle = true;
        self.complete_if_idle(generation)
    }

    pub fn complete_if_idle(&mut self, generation: u64) -> bool {
        if !self.accepts_callback(generation) || !self.reply_closed || !self.native_idle {
            return false;
        }
        self.active_reply_id = None;
        self.pending.clear();
        true
    }

    fn ensure_reply(&self, reply_id: &str) -> Result<(), VoiceReplyError> {
        if !self.enabled {
            return Err(VoiceReplyError::Disabled);
        }
        if self.reply_closed || self.active_reply_id.as_deref() != Some(reply_id) {
            return Err(VoiceReplyError::ReplyMismatch);
        }
        Ok(())
    }

    fn emit_ready(&mut self, requests: &mut Vec<SpeakRequest>, boundary: bool) {
        let long = self.pending.chars().count() >= MAX_SEGMENT_CHARS;
        if !boundary && !long {
            return;
        }
        let raw = std::mem::take(&mut self.pending);
        let spoken = speakable_text(&raw);
        if spoken.is_empty() {
            return;
        }
        requests.push(SpeakRequest {
            generation: self.generation,
            sequence: self.next_sequence,
            text: spoken,
        });
        self.next_sequence += 1;
        self.native_idle = false;
    }

    fn invalidate(&mut self) {
        self.generation = self.generation.wrapping_add(1).max(1);
        self.active_reply_id = None;
        self.pending.clear();
        self.next_sequence = 0;
        self.total_chars = 0;
        self.in_code_block = false;
        self.backtick_run = 0;
        self.reply_closed = false;
        self.native_idle = true;
    }
}

fn speakable_text(raw: &str) -> String {
    let mut result = String::new();
    for token in raw.split_whitespace() {
        let value = if token.starts_with("https://") || token.starts_with("http://") {
            "链接"
        } else {
            token
        };
        for character in value.chars() {
            if !matches!(character, '#' | '*' | '_' | '~' | '>' | '[' | ']' | '(' | ')' | '|') {
                result.push(character);
            }
        }
        result.push(' ');
    }
    result.trim().to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disabled_session_never_accepts_reply() {
        let mut session = VoiceReplySession::default();
        assert_eq!(session.begin("reply-1"), Err(VoiceReplyError::Disabled));
        assert!(session.push_delta("reply-1", "内容。").is_err());
    }

    #[test]
    fn stable_sentences_are_numbered_and_tail_flushes() {
        let mut session = VoiceReplySession::new(true);
        let generation = session.begin("reply-1").unwrap();
        let first = session.push_delta("reply-1", "第一句。第二").unwrap();
        assert_eq!(first, vec![SpeakRequest { generation, sequence: 0, text: "第一句。".into() }]);
        assert!(session.push_delta("reply-1", "句").unwrap().is_empty());
        assert_eq!(
            session.finish("reply-1").unwrap(),
            vec![SpeakRequest { generation, sequence: 1, text: "第二句".into() }]
        );
    }

    #[test]
    fn markdown_code_and_links_do_not_leak_raw_syntax() {
        let mut session = VoiceReplySession::new(true);
        session.begin("reply-1").unwrap();
        let requests = session
            .push_delta("reply-1", "**结果**见 https://example.com。```secret\n```完成！")
            .unwrap();
        let spoken = requests.into_iter().map(|request| request.text).collect::<String>();
        assert!(!spoken.contains("https://"));
        assert!(!spoken.contains("secret"));
        assert!(!spoken.contains('*'));
        assert!(spoken.contains("链接"));
        assert!(spoken.contains("代码内容已省略"));
    }

    #[test]
    fn stop_invalidates_late_native_callbacks() {
        let mut session = VoiceReplySession::new(true);
        let old = session.begin("reply-1").unwrap();
        assert!(session.accepts_callback(old));
        let stopped = session.stop();
        assert_ne!(old, stopped);
        assert!(!session.accepts_callback(old));
        let current = session.begin("reply-2").unwrap();
        assert!(!session.accepts_callback(old));
        assert!(session.accepts_callback(current));
    }

    #[test]
    fn completion_only_clears_a_closed_current_reply() {
        let mut session = VoiceReplySession::new(true);
        let generation = session.begin("reply-1").unwrap();
        assert!(!session.complete_if_idle(generation));
        session.finish("reply-1").unwrap();
        assert!(session.complete_if_idle(generation));
        assert!(!session.accepts_callback(generation));
    }

    #[test]
    fn native_idle_before_reply_terminal_completes_after_finish() {
        let mut session = VoiceReplySession::new(true);
        let generation = session.begin("reply-1").unwrap();
        assert_eq!(session.push_delta("reply-1", "一句。").unwrap().len(), 1);
        assert!(!session.native_idle(generation));
        session.finish("reply-1").unwrap();
        assert!(session.complete_if_idle(generation));
    }

    #[test]
    fn preview_uses_a_new_bounded_generation_and_closes_it() {
        let mut session = VoiceReplySession::new(true);
        session.begin("reply-1").unwrap();
        let previous = session.generation();
        let requests = session.preview("Yonder 本地语音播放正常。").unwrap();
        assert_eq!(requests.len(), 1);
        assert_ne!(requests[0].generation, previous);
        assert_eq!(requests[0].sequence, 0);
        assert_eq!(requests[0].text, "Yonder 本地语音播放正常。");
        assert!(session.native_idle(requests[0].generation));
    }
}
