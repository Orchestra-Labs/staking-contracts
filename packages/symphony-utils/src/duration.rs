use cw_utils::Duration;
use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum UnboundingDurationError {
    #[error("Invalid unstaking duration, unstaking duration cannot be 0")]
    InvalidUnboundingDuration {},
}

pub fn validate_duration(duration: Option<Duration>) -> Result<(), UnboundingDurationError> {
    if let Some(unbounding_duration) = duration {
        match unbounding_duration {
            Duration::Height(height) => {
                if height == 0 {
                    return Err(UnboundingDurationError::InvalidUnboundingDuration {});
                }
            }
            Duration::Time(time) => {
                if time == 0 {
                    return Err(UnboundingDurationError::InvalidUnboundingDuration {});
                }
            }
        }
    }

    Ok(())
}