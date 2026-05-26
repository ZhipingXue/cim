namespace Cim.ControlApp.Common.StateMachine;

/// <summary>
/// Process state enum for chamber modules.
/// </summary>
public enum ProcessState
{
    Idle = 0,
    Ready = 1,
    Processing = 2,
    Complete = 3,
    Error = 4
}

/// <summary>
/// Process state control interface for chamber modules.
/// </summary>
public interface IProcessStateControl : IStateControl<ProcessState>
{
    Task<StateTransitionResult<ProcessState, StateTransitionErrorCode>> StartProcessingAsync(string recipeName, CancellationToken ct = default);
    Task<StateTransitionResult<ProcessState, StateTransitionErrorCode>> PauseProcessingAsync(CancellationToken ct = default);
    Task<StateTransitionResult<ProcessState, StateTransitionErrorCode>> ResumeProcessingAsync(CancellationToken ct = default);
    Task<StateTransitionResult<ProcessState, StateTransitionErrorCode>> AbortProcessingAsync(CancellationToken ct = default);
    Task<StateTransitionResult<ProcessState, StateTransitionErrorCode>> CompleteProcessingAsync(CancellationToken ct = default);
}

/// <summary>
/// Abstract base for process state control. Override action methods for custom logic.
/// </summary>
public abstract class ProcessStateControlBase : StateControlBase<ProcessState>, IProcessStateControl
{
    protected ProcessStateControlBase() : base(ProcessState.Idle, BuildTransitions()) { }

    private static Dictionary<ProcessState, HashSet<ProcessState>> BuildTransitions()
    {
        return new Dictionary<ProcessState, HashSet<ProcessState>>
        {
            [ProcessState.Idle] = new() { ProcessState.Ready, ProcessState.Error },
            [ProcessState.Ready] = new() { ProcessState.Processing, ProcessState.Idle, ProcessState.Error },
            [ProcessState.Processing] = new() { ProcessState.Complete, ProcessState.Error },
            [ProcessState.Complete] = new() { ProcessState.Idle, ProcessState.Error },
            [ProcessState.Error] = new() { ProcessState.Idle }
        };
    }

    public virtual async Task<StateTransitionResult<ProcessState, StateTransitionErrorCode>> StartProcessingAsync(string recipeName, CancellationToken ct = default)
    {
        var result1 = await TryTransitionToAsync(ProcessState.Ready, ct);
        if (!result1.IsSuccess) return result1;

        var actionResult = await OnStartProcessingAsync(recipeName, ct);
        if (!actionResult.IsSuccess)
        {
            // Rollback to idle on action failure
            await TryTransitionToAsync(ProcessState.Idle, ct);
            return StateTransitionResult<ProcessState, StateTransitionErrorCode>.Failure(
                StateTransitionErrorCode.PreConditionNotMet, ProcessState.Ready, ProcessState.Processing,
                actionResult.ErrorMessage ?? "Action failed", actionResult.Details);
        }

        var result2 = await TryTransitionToAsync(ProcessState.Processing, ct);
        return result2;
    }

    public virtual async Task<StateTransitionResult<ProcessState, StateTransitionErrorCode>> PauseProcessingAsync(CancellationToken ct = default)
    {
        var actionResult = await OnPauseProcessingAsync(ct);
        if (!actionResult.IsSuccess)
        {
            return StateTransitionResult<ProcessState, StateTransitionErrorCode>.Failure(
                StateTransitionErrorCode.InternalError, ProcessState.Processing, ProcessState.Processing,
                actionResult.ErrorMessage ?? "Pause failed", actionResult.Details);
        }

        return StateTransitionResult<ProcessState, StateTransitionErrorCode>.Success(
            ProcessState.Processing, ProcessState.Processing,
            new Dictionary<string, string> { ["action"] = "Paused processing" });
    }

    public virtual async Task<StateTransitionResult<ProcessState, StateTransitionErrorCode>> ResumeProcessingAsync(CancellationToken ct = default)
    {
        var actionResult = await OnResumeProcessingAsync(ct);
        if (!actionResult.IsSuccess)
        {
            return StateTransitionResult<ProcessState, StateTransitionErrorCode>.Failure(
                StateTransitionErrorCode.InternalError, ProcessState.Processing, ProcessState.Processing,
                actionResult.ErrorMessage ?? "Resume failed", actionResult.Details);
        }

        return StateTransitionResult<ProcessState, StateTransitionErrorCode>.Success(
            ProcessState.Processing, ProcessState.Processing,
            new Dictionary<string, string> { ["action"] = "Resumed processing" });
    }

    public virtual async Task<StateTransitionResult<ProcessState, StateTransitionErrorCode>> AbortProcessingAsync(CancellationToken ct = default)
    {
        var actionResult = await OnAbortProcessingAsync(ct);
        if (!actionResult.IsSuccess)
        {
            return StateTransitionResult<ProcessState, StateTransitionErrorCode>.Failure(
                StateTransitionErrorCode.InternalError, ProcessState.Processing, ProcessState.Idle,
                actionResult.ErrorMessage ?? "Abort failed", actionResult.Details);
        }

        return await TryTransitionToAsync(ProcessState.Idle, ct);
    }

    public virtual async Task<StateTransitionResult<ProcessState, StateTransitionErrorCode>> CompleteProcessingAsync(CancellationToken ct = default)
    {
        var actionResult = await OnCompleteProcessingAsync(ct);
        if (!actionResult.IsSuccess)
        {
            return StateTransitionResult<ProcessState, StateTransitionErrorCode>.Failure(
                StateTransitionErrorCode.PostConditionFailed, ProcessState.Processing, ProcessState.Complete,
                actionResult.ErrorMessage ?? "Complete failed", actionResult.Details);
        }

        return await TryTransitionToAsync(ProcessState.Complete, ct);
    }

    // Override points for derived classes - return ActionResult
    protected virtual Task<ActionResult> OnStartProcessingAsync(string recipeName, CancellationToken ct)
        => Task.FromResult(ActionResult.Success("recipe", recipeName));
    protected virtual Task<ActionResult> OnPauseProcessingAsync(CancellationToken ct)
        => Task.FromResult(ActionResult.Success());
    protected virtual Task<ActionResult> OnResumeProcessingAsync(CancellationToken ct)
        => Task.FromResult(ActionResult.Success());
    protected virtual Task<ActionResult> OnAbortProcessingAsync(CancellationToken ct)
        => Task.FromResult(ActionResult.Success());
    protected virtual Task<ActionResult> OnCompleteProcessingAsync(CancellationToken ct)
        => Task.FromResult(ActionResult.Success());
}

/// <summary>
/// Default process state control implementation.
/// </summary>
public class ProcessStateControl : ProcessStateControlBase
{
    // Default implementation - all logic uses base class no-op overrides
}
