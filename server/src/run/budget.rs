use crate::domain::run::{RunBudgetLimits, RunBudgetUsage};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BudgetCheckpoint {
    BeforeTurn,
    DuringTurn,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BudgetDecision {
    AllowTurn,
    StopBeforeTurn(BudgetLimit),
    InterruptTurn(BudgetLimit),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BudgetLimit {
    ModelTurns,
    ElapsedTime,
    Tokens,
}

pub fn evaluate_budget(
    limits: RunBudgetLimits,
    usage: RunBudgetUsage,
    checkpoint: BudgetCheckpoint,
) -> BudgetDecision {
    let exceeded = match checkpoint {
        BudgetCheckpoint::BeforeTurn => [
            (
                usage.model_turns_started >= limits.max_model_turns_per_run,
                BudgetLimit::ModelTurns,
            ),
            (
                usage.elapsed_seconds >= limits.max_elapsed_seconds_per_run,
                BudgetLimit::ElapsedTime,
            ),
            (
                usage.reported_tokens >= limits.max_tokens_per_run,
                BudgetLimit::Tokens,
            ),
        ],
        BudgetCheckpoint::DuringTurn => [
            (false, BudgetLimit::ModelTurns),
            (
                usage.elapsed_seconds >= limits.max_elapsed_seconds_per_run,
                BudgetLimit::ElapsedTime,
            ),
            (
                usage.reported_tokens >= limits.max_tokens_per_run,
                BudgetLimit::Tokens,
            ),
        ],
    }
    .into_iter()
    .find_map(|(is_exceeded, limit)| is_exceeded.then_some(limit));

    match (checkpoint, exceeded) {
        (_, None) => BudgetDecision::AllowTurn,
        (BudgetCheckpoint::BeforeTurn, Some(limit)) => BudgetDecision::StopBeforeTurn(limit),
        (BudgetCheckpoint::DuringTurn, Some(limit)) => BudgetDecision::InterruptTurn(limit),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::run::BudgetConfigError;

    fn limits() -> RunBudgetLimits {
        RunBudgetLimits {
            max_model_turns_per_run: 4,
            max_elapsed_seconds_per_run: 600,
            max_tokens_per_run: 50_000,
        }
    }

    #[test]
    fn budget_override_can_only_tighten_project_values() {
        let project = limits();
        let tightened = RunBudgetLimits {
            max_model_turns_per_run: 2,
            max_elapsed_seconds_per_run: 300,
            max_tokens_per_run: 20_000,
        };

        assert_eq!(project.tighten_with(Some(tightened)), Ok(tightened));
        assert_eq!(
            project.tighten_with(Some(RunBudgetLimits {
                max_tokens_per_run: 60_000,
                ..tightened
            })),
            Err(BudgetConfigError::OverrideRaisesProjectLimit),
        );
    }

    #[test]
    fn model_turn_and_elapsed_limits_stop_before_another_turn() {
        let limits = limits();

        assert_eq!(
            evaluate_budget(
                limits,
                RunBudgetUsage {
                    model_turns_started: 4,
                    ..RunBudgetUsage::default()
                },
                BudgetCheckpoint::BeforeTurn,
            ),
            BudgetDecision::StopBeforeTurn(BudgetLimit::ModelTurns),
        );
        assert_eq!(
            evaluate_budget(
                limits,
                RunBudgetUsage {
                    elapsed_seconds: 600,
                    ..RunBudgetUsage::default()
                },
                BudgetCheckpoint::BeforeTurn,
            ),
            BudgetDecision::StopBeforeTurn(BudgetLimit::ElapsedTime),
        );
    }

    #[test]
    fn observed_token_excess_interrupts_current_turn_and_blocks_next() {
        let limits = limits();
        let usage = RunBudgetUsage {
            reported_tokens: 50_001,
            ..RunBudgetUsage::default()
        };

        assert_eq!(
            evaluate_budget(limits, usage, BudgetCheckpoint::DuringTurn),
            BudgetDecision::InterruptTurn(BudgetLimit::Tokens),
        );
        assert_eq!(
            evaluate_budget(limits, usage, BudgetCheckpoint::BeforeTurn),
            BudgetDecision::StopBeforeTurn(BudgetLimit::Tokens),
        );
    }
}
