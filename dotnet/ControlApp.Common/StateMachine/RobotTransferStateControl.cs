namespace Cim.ControlApp.Common.StateMachine;

/// <summary>
/// Robot transfer state enum.
/// </summary>
public enum RobotTransferState
{
    Idle = 0,
    Picking = 1,
    Moving = 2,
    Placing = 3,
    Aligning = 4,
    Error = 5
}

/// <summary>
/// Robot transfer state control interface.
/// </summary>
public interface IRobotTransferStateControl : IStateControl<RobotTransferState>
{
    Task<StateTransitionResult<RobotTransferState, StateTransitionErrorCode>> PickAsync(string sourceLocation, CancellationToken ct = default);
    Task<StateTransitionResult<RobotTransferState, StateTransitionErrorCode>> PlaceAsync(string destinationLocation, CancellationToken ct = default);
    Task<StateTransitionResult<RobotTransferState, StateTransitionErrorCode>> MoveAsync(string destinationLocation, CancellationToken ct = default);
}

/// <summary>
/// Abstract base for robot transfer state control. Override action methods for custom logic.
/// </summary>
public abstract class RobotTransferStateControlBase : StateControlBase<RobotTransferState>, IRobotTransferStateControl
{
    protected RobotTransferStateControlBase() : base(RobotTransferState.Idle, BuildTransitions()) { }

    private static Dictionary<RobotTransferState, HashSet<RobotTransferState>> BuildTransitions()
    {
        return new Dictionary<RobotTransferState, HashSet<RobotTransferState>>
        {
            [RobotTransferState.Idle] = new() { RobotTransferState.Picking, RobotTransferState.Moving, RobotTransferState.Error },
            [RobotTransferState.Picking] = new() { RobotTransferState.Moving, RobotTransferState.Idle, RobotTransferState.Error },
            [RobotTransferState.Moving] = new() { RobotTransferState.Placing, RobotTransferState.Idle, RobotTransferState.Error },
            [RobotTransferState.Placing] = new() { RobotTransferState.Idle, RobotTransferState.Error },
            [RobotTransferState.Aligning] = new() { RobotTransferState.Idle, RobotTransferState.Error },
            [RobotTransferState.Error] = new() { RobotTransferState.Idle }
        };
    }

    public virtual async Task<StateTransitionResult<RobotTransferState, StateTransitionErrorCode>> PickAsync(string sourceLocation, CancellationToken ct = default)
    {
        var actionResult = await OnPickAsync(sourceLocation, ct);
        if (!actionResult.IsSuccess)
        {
            return StateTransitionResult<RobotTransferState, StateTransitionErrorCode>.Failure(
                StateTransitionErrorCode.PreConditionNotMet, RobotTransferState.Idle, RobotTransferState.Picking,
                actionResult.ErrorMessage ?? "Pick failed", actionResult.Details);
        }

        return await TryTransitionToAsync(RobotTransferState.Picking, ct);
    }

    public virtual async Task<StateTransitionResult<RobotTransferState, StateTransitionErrorCode>> PlaceAsync(string destinationLocation, CancellationToken ct = default)
    {
        var actionResult = await OnPlaceAsync(destinationLocation, ct);
        if (!actionResult.IsSuccess)
        {
            return StateTransitionResult<RobotTransferState, StateTransitionErrorCode>.Failure(
                StateTransitionErrorCode.PreConditionNotMet, RobotTransferState.Moving, RobotTransferState.Placing,
                actionResult.ErrorMessage ?? "Place failed", actionResult.Details);
        }

        return await TryTransitionToAsync(RobotTransferState.Placing, ct);
    }

    public virtual async Task<StateTransitionResult<RobotTransferState, StateTransitionErrorCode>> MoveAsync(string destinationLocation, CancellationToken ct = default)
    {
        var actionResult = await OnMoveAsync(destinationLocation, ct);
        if (!actionResult.IsSuccess)
        {
            return StateTransitionResult<RobotTransferState, StateTransitionErrorCode>.Failure(
                StateTransitionErrorCode.PreConditionNotMet, RobotTransferState.Idle, RobotTransferState.Moving,
                actionResult.ErrorMessage ?? "Move failed", actionResult.Details);
        }

        return await TryTransitionToAsync(RobotTransferState.Moving, ct);
    }

    // Override points for derived classes - return ActionResult
    protected virtual Task<ActionResult> OnPickAsync(string sourceLocation, CancellationToken ct)
        => Task.FromResult(ActionResult.Success("source", sourceLocation));
    protected virtual Task<ActionResult> OnPlaceAsync(string destinationLocation, CancellationToken ct)
        => Task.FromResult(ActionResult.Success("destination", destinationLocation));
    protected virtual Task<ActionResult> OnMoveAsync(string destinationLocation, CancellationToken ct)
        => Task.FromResult(ActionResult.Success("destination", destinationLocation));
}

/// <summary>
/// Default robot transfer state control implementation.
/// </summary>
public class RobotTransferStateControl : RobotTransferStateControlBase
{
    // Default implementation - all logic uses base class no-op overrides
}
