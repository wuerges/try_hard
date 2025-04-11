/// A hard result contains a hard error in its [Err] variant, and a [SoftResult] in its [Ok] variant.
/// A hard error is a catastrophic failure, that should be avoided at all costs.
pub type HardResult<T, SoftError, HardError> = Result<SoftResult<T, SoftError>, HardError>;

/// A [SoftResult], should only contain errors if these errors are benign, and can be presented to the user as a valid response.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[must_use]
pub enum SoftResult<T, E> {
    Ok(T),
    SoftErr(E),
}

macro_rules! try_soft {
    ($e:expr) => {
        match $e {
            SoftResult::Ok(t) => t,
            SoftResult::SoftErr(e) => return Result::Ok(SoftResult::SoftErr(e)),
        }
    };
}

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
        hard_result: HardResult<(), SoftError, HardError>,
    ) -> HardResult<(), SoftError, HardError> {
        Ok(SoftResult::Ok(try_hard!(hard_result)))
    }

    #[instrument(err)]
    fn tries_soft(soft_result: SoftResult<(), SoftError>) -> HardResult<(), SoftError, HardError> {
        Ok(SoftResult::Ok(try_soft!(soft_result)))
    }

    #[rstest]
    #[case(Ok(SoftResult::Ok(())))]
    #[case(Ok(SoftResult::SoftErr(SoftError)))]
    #[case(Err(HardError))]
    fn check_try_hard(#[case] hard_result: HardResult<(), SoftError, HardError>) {
        let result = tries_hard(hard_result.clone());
        assert_eq!(result, hard_result)
    }

    #[rstest]
    #[case(SoftResult::Ok(()))]
    #[case(SoftResult::SoftErr(SoftError))]
    fn check_try_soft(#[case] soft_result: SoftResult<(), SoftError>) {
        let result = tries_soft(soft_result.clone());
        assert_eq!(result, Ok(soft_result))
    }
}
