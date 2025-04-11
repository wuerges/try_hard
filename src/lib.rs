//! Provides the [MalleableResult] and [SoftResult] types, and the [try_hard] and [try_soft] macros.
//! Support the `tracing` crate!
//!
//! A [MalleableResult] distinguishes errors in two categories:
//! - Soft errors:
//!     These are "benign" error types that shouldn't cause your application to stop.
//!     Think of 404 errors, or any other error that was caused by the user, and not by your application.
//!     Soft errors won't trigger `error` events when used with the `#[instrument(err)]` `tracing` macro.
//!
//! - Hard errors:
//!     These are bad.
//!     Hard errors are in general not fault of the user, and the user is hopeless without our intervention.
//!     Hard errors must be monitored.
//!     Hard errors will result in `error` events when used with the `#[instrument(err)]` `tracing macro.`
//!

/// A hard result contains a hard error in its [Err] variant, and a [SoftResult] in its [Ok] variant.
/// A hard error is a catastrophic failure, that should be avoided at all costs.
pub type MalleableResult<T, SoftError, HardError> = Result<SoftResult<T, SoftError>, HardError>;

/// A [SoftResult], should only contain errors if these errors are benign, and can be presented to the user as a valid response.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[must_use]
pub enum SoftResult<T, E> {
    /// The Ok variant should be used like the core result [Ok].
    Ok(T),
    /// The [SoftResult::SoftErr] variant should be reserved to soft errors.
    /// These are benign errors that can be returned to the user, such as a 404 in a web application.
    SoftErr(E),
}

#[macro_export]
/// The `try_soft` macro does the job of the `?` operator: extract the [SoftResult::Ok] Value, without short-circuiting.
/// It will short-circuit in case of a [SoftResult::SoftErr], returning a `MalleableResult::Ok(SoftResult::SoftErr(_))`.
macro_rules! try_soft {
    ($e:expr) => {
        match $e {
            SoftResult::Ok(t) => t,
            SoftResult::SoftErr(e) => return Result::Ok(SoftResult::SoftErr(e)),
        }
    };
}

#[macro_export]
/// The `try_soft` macro does the job of the `?` operator: extract the [SoftResult::Ok] Value, without short-circuiting.
/// It will short-circuit case or errors:
/// - In case of [SoftResult::SoftErr], returning a `MalleableResult::Ok(SoftResult::SoftErr(_))`.
/// - In case of [MalleableResult]::Err, returning a `MalleableResult::Err(_)`.
macro_rules! try_hard {
    ($e:expr) => {
        match $e {
            Result::Ok(t) => try_soft!(t),
            Result::Err(e) => return Result::Err(e),
        }
    };
}

#[cfg(test)]
mod tests {
    use rstest::rstest;
    use tracing::instrument;

    use super::*;

    #[derive(Debug, thiserror::Error, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
    #[error("a soft error")]
    struct SoftError;

    #[derive(Debug, thiserror::Error, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
    #[error("a real dangerous error")]
    struct HardError;

    #[instrument(err)]
    fn tries_hard(
        hard_result: MalleableResult<(), SoftError, HardError>,
        has_skipped: &mut bool,
    ) -> MalleableResult<(), SoftError, HardError> {
        let x = try_hard!(hard_result);
        *has_skipped = false;
        Ok(SoftResult::Ok(x))
    }

    #[instrument(err)]
    fn tries_soft(
        soft_result: SoftResult<(), SoftError>,
    ) -> MalleableResult<(), SoftError, HardError> {
        Ok(SoftResult::Ok(try_soft!(soft_result)))
    }

    #[rstest]
    #[case(Ok(SoftResult::Ok(())), false)]
    #[case(Ok(SoftResult::SoftErr(SoftError)), true)]
    #[case(Err(HardError), true)]
    fn check_try_hard(
        #[case] hard_result: MalleableResult<(), SoftError, HardError>,
        #[case] expected_skip: bool,
    ) {
        tracing_subscriber::fmt::try_init().ok();

        let mut has_skipped = true;
        let result = tries_hard(hard_result.clone(), &mut has_skipped);
        assert_eq!(result, hard_result);
        assert_eq!(has_skipped, expected_skip);
    }

    #[rstest]
    #[case(SoftResult::Ok(()))]
    #[case(SoftResult::SoftErr(SoftError))]
    fn check_try_soft(#[case] soft_result: SoftResult<(), SoftError>) {
        tracing_subscriber::fmt::try_init().ok();

        let result = tries_soft(soft_result.clone());
        assert_eq!(result, Ok(soft_result))
    }
}
