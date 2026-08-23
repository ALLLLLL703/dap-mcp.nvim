use crate::error::SidecarError;

/// Execution status for the single active debug session.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DebugStatus {
    /// No active debug session.
    Inactive,
    /// Debuggee is running.
    Running,
    /// Debuggee is stopped with an actionable frame.
    Stopped,
}

impl DebugStatus {
    /// Returns a stable status name for errors and responses.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Inactive => "inactive",
            Self::Running => "running",
            Self::Stopped => "stopped",
        }
    }
}

/// Current single-session debug context.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DebugState {
    /// Current execution status.
    pub status: DebugStatus,
    /// Named launch configuration for an active session.
    pub configuration_name: Option<String>,
    /// Current DAP thread identifier when stopped.
    pub thread_id: Option<i64>,
    /// Current DAP frame identifier when stopped.
    pub frame_id: Option<i64>,
}

impl Default for DebugState {
    /// Creates an inactive debug context.
    fn default() -> Self {
        Self {
            status: DebugStatus::Inactive,
            configuration_name: None,
            thread_id: None,
            frame_id: None,
        }
    }
}

impl DebugState {
    /// Starts a new named debug session when no session is active.
    pub fn start(&mut self, configuration_name: String) -> Result<(), SidecarError> {
        if self.status != DebugStatus::Inactive {
            return Err(SidecarError::InvalidDebugTransition {
                from: self.status.as_str(),
                to: "running",
            });
        }
        self.status = DebugStatus::Running;
        self.configuration_name = Some(configuration_name);
        Ok(())
    }

    /// Records an actionable stopped frame.
    pub fn mark_stopped(
        &mut self,
        thread_id: Option<i64>,
        frame_id: Option<i64>,
    ) -> Result<(), SidecarError> {
        if self.status == DebugStatus::Inactive {
            return Err(SidecarError::InvalidDebugTransition {
                from: self.status.as_str(),
                to: "stopped",
            });
        }
        self.status = DebugStatus::Stopped;
        self.thread_id = thread_id;
        self.frame_id = frame_id;
        Ok(())
    }

    /// Records continued execution and clears stale frame context.
    pub fn mark_running(&mut self) -> Result<(), SidecarError> {
        if self.status != DebugStatus::Stopped {
            return Err(SidecarError::InvalidDebugTransition {
                from: self.status.as_str(),
                to: "running",
            });
        }
        self.status = DebugStatus::Running;
        self.thread_id = None;
        self.frame_id = None;
        Ok(())
    }

    /// Clears all state after termination.
    pub fn terminate(&mut self) {
        *self = Self::default();
    }
}

#[cfg(test)]
mod tests {
    use super::{DebugState, DebugStatus};

    /// Enforces a single active session and valid stop/continue transitions.
    #[test]
    fn follows_single_session_transitions() {
        let mut state = DebugState::default();
        assert!(state.start("Launch app".to_owned()).is_ok());
        assert!(state.start("Second".to_owned()).is_err());
        assert!(state.mark_stopped(Some(3), Some(9)).is_ok());
        assert_eq!(state.status, DebugStatus::Stopped);
        assert!(state.mark_running().is_ok());
        assert_eq!(state.frame_id, None);
        state.terminate();
        assert_eq!(state, DebugState::default());
    }

    /// Rejects stopped state without an active session.
    #[test]
    fn rejects_stop_without_session() {
        let mut state = DebugState::default();
        assert!(state.mark_stopped(Some(1), Some(1)).is_err());
    }
}
