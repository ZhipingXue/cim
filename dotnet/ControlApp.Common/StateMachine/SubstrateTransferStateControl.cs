namespace Cim.ControlApp.Common.StateMachine;

/// <summary>
/// Substrate transfer state for load/unload operations.
/// </summary>
public enum SubstrateTransferState
{
    Idle = 0,
    PrepareLoad = 1,
    Loading = 2,
    LoadSuccess = 3,
    LoadFailure = 4,
    PrepareUnload = 5,
    Unloading = 6,
    UnloadSuccess = 7,
    UnloadFailure = 8,
    Error = 9
}

/// <summary>
/// Context for substrate transfer operations.
/// </summary>
public class SubstrateTransferContext
{
    public string? SubstrateId { get; set; }
    public string? SourceLocation { get; set; }
    public string? DestinationLocation { get; set; }
    public DateTimeOffset? StartTime { get; set; }
    public DateTimeOffset? EndTime { get; set; }
    public string? ErrorMessage { get; set; }
    public Dictionary<string, string> Metadata { get; set; } = new();
}

/// <summary>
/// Interface for substrate transfer state control.
/// </summary>
public interface ISubstrateTransferStateControl : IStateControl<SubstrateTransferState, SubstrateTransferContext>
{
    Task<StateTransitionResult<SubstrateTransferState, StateTransitionErrorCode>> PrepareLoadAsync(string substrateId, string sourceLocation, CancellationToken ct = default);
    Task<StateTransitionResult<SubstrateTransferState, StateTransitionErrorCode>> ConfirmLoadSuccessAsync(CancellationToken ct = default);
    Task<StateTransitionResult<SubstrateTransferState, StateTransitionErrorCode>> ConfirmLoadFailureAsync(string errorMessage, CancellationToken ct = default);
    Task<StateTransitionResult<SubstrateTransferState, StateTransitionErrorCode>> PrepareUnloadAsync(string substrateId, string destinationLocation, CancellationToken ct = default);
    Task<StateTransitionResult<SubstrateTransferState, StateTransitionErrorCode>> ConfirmUnloadSuccessAsync(CancellationToken ct = default);
    Task<StateTransitionResult<SubstrateTransferState, StateTransitionErrorCode>> ConfirmUnloadFailureAsync(string errorMessage, CancellationToken ct = default);
    Task<StateTransitionResult<SubstrateTransferState, StateTransitionErrorCode>> ResetAsync(CancellationToken ct = default);
}

/// <summary>
/// Abstract base for substrate transfer state control. Override action methods for custom logic.
/// </summary>
public abstract class SubstrateTransferStateControlBase : StateControlBase<SubstrateTransferState, SubstrateTransferContext>, ISubstrateTransferStateControl
{
    protected SubstrateTransferStateControlBase() : base(SubstrateTransferState.Idle, BuildTransitions()) { }

    private static Dictionary<SubstrateTransferState, HashSet<SubstrateTransferState>> BuildTransitions()
    {
        return new Dictionary<SubstrateTransferState, HashSet<SubstrateTransferState>>
        {
            [SubstrateTransferState.Idle] = new() { SubstrateTransferState.PrepareLoad, SubstrateTransferState.PrepareUnload, SubstrateTransferState.Error },
            [SubstrateTransferState.PrepareLoad] = new() { SubstrateTransferState.Loading, SubstrateTransferState.Idle, SubstrateTransferState.Error },
            [SubstrateTransferState.Loading] = new() { SubstrateTransferState.LoadSuccess, SubstrateTransferState.LoadFailure, SubstrateTransferState.Error },
            [SubstrateTransferState.LoadSuccess] = new() { SubstrateTransferState.Idle, SubstrateTransferState.PrepareUnload, SubstrateTransferState.Error },
            [SubstrateTransferState.LoadFailure] = new() { SubstrateTransferState.Idle, SubstrateTransferState.PrepareLoad, SubstrateTransferState.Error },
            [SubstrateTransferState.PrepareUnload] = new() { SubstrateTransferState.Unloading, SubstrateTransferState.Idle, SubstrateTransferState.Error },
            [SubstrateTransferState.Unloading] = new() { SubstrateTransferState.UnloadSuccess, SubstrateTransferState.UnloadFailure, SubstrateTransferState.Error },
            [SubstrateTransferState.UnloadSuccess] = new() { SubstrateTransferState.Idle, SubstrateTransferState.PrepareLoad, SubstrateTransferState.Error },
            [SubstrateTransferState.UnloadFailure] = new() { SubstrateTransferState.Idle, SubstrateTransferState.PrepareUnload, SubstrateTransferState.Error },
            [SubstrateTransferState.Error] = new() { SubstrateTransferState.Idle }
        };
    }

    public virtual async Task<StateTransitionResult<SubstrateTransferState, StateTransitionErrorCode>> PrepareLoadAsync(string substrateId, string sourceLocation, CancellationToken ct = default)
    {
        var context = new SubstrateTransferContext
        {
            SubstrateId = substrateId,
            SourceLocation = sourceLocation,
            StartTime = DateTimeOffset.UtcNow
        };
        await OnPrepareLoadAsync(substrateId, sourceLocation, ct);
        return await TryTransitionToAsync(SubstrateTransferState.PrepareLoad, context, ct);
    }

    public virtual async Task<StateTransitionResult<SubstrateTransferState, StateTransitionErrorCode>> ConfirmLoadSuccessAsync(CancellationToken ct = default)
    {
        var context = Context ?? new SubstrateTransferContext();
        context.EndTime = DateTimeOffset.UtcNow;
        await OnConfirmLoadSuccessAsync(ct);
        return await TryTransitionToAsync(SubstrateTransferState.LoadSuccess, context, ct);
    }

    public virtual async Task<StateTransitionResult<SubstrateTransferState, StateTransitionErrorCode>> ConfirmLoadFailureAsync(string errorMessage, CancellationToken ct = default)
    {
        var context = Context ?? new SubstrateTransferContext();
        context.EndTime = DateTimeOffset.UtcNow;
        context.ErrorMessage = errorMessage;
        await OnConfirmLoadFailureAsync(errorMessage, ct);
        return await TryTransitionToAsync(SubstrateTransferState.LoadFailure, context, ct);
    }

    public virtual async Task<StateTransitionResult<SubstrateTransferState, StateTransitionErrorCode>> PrepareUnloadAsync(string substrateId, string destinationLocation, CancellationToken ct = default)
    {
        var context = new SubstrateTransferContext
        {
            SubstrateId = substrateId,
            DestinationLocation = destinationLocation,
            StartTime = DateTimeOffset.UtcNow
        };
        await OnPrepareUnloadAsync(substrateId, destinationLocation, ct);
        return await TryTransitionToAsync(SubstrateTransferState.PrepareUnload, context, ct);
    }

    public virtual async Task<StateTransitionResult<SubstrateTransferState, StateTransitionErrorCode>> ConfirmUnloadSuccessAsync(CancellationToken ct = default)
    {
        var context = Context ?? new SubstrateTransferContext();
        context.EndTime = DateTimeOffset.UtcNow;
        await OnConfirmUnloadSuccessAsync(ct);
        return await TryTransitionToAsync(SubstrateTransferState.UnloadSuccess, context, ct);
    }

    public virtual async Task<StateTransitionResult<SubstrateTransferState, StateTransitionErrorCode>> ConfirmUnloadFailureAsync(string errorMessage, CancellationToken ct = default)
    {
        var context = Context ?? new SubstrateTransferContext();
        context.EndTime = DateTimeOffset.UtcNow;
        context.ErrorMessage = errorMessage;
        await OnConfirmUnloadFailureAsync(errorMessage, ct);
        return await TryTransitionToAsync(SubstrateTransferState.UnloadFailure, context, ct);
    }

    public virtual async Task<StateTransitionResult<SubstrateTransferState, StateTransitionErrorCode>> ResetAsync(CancellationToken ct = default)
    {
        await OnResetAsync(ct);
        return await TryTransitionToAsync(SubstrateTransferState.Idle, null, ct);
    }

    // Override points for derived classes
    protected virtual Task OnPrepareLoadAsync(string substrateId, string sourceLocation, CancellationToken ct) => Task.CompletedTask;
    protected virtual Task OnConfirmLoadSuccessAsync(CancellationToken ct) => Task.CompletedTask;
    protected virtual Task OnConfirmLoadFailureAsync(string errorMessage, CancellationToken ct) => Task.CompletedTask;
    protected virtual Task OnPrepareUnloadAsync(string substrateId, string destinationLocation, CancellationToken ct) => Task.CompletedTask;
    protected virtual Task OnConfirmUnloadSuccessAsync(CancellationToken ct) => Task.CompletedTask;
    protected virtual Task OnConfirmUnloadFailureAsync(string errorMessage, CancellationToken ct) => Task.CompletedTask;
    protected virtual Task OnResetAsync(CancellationToken ct) => Task.CompletedTask;
}

/// <summary>
/// Default substrate transfer state control implementation.
/// </summary>
public class SubstrateTransferStateControl : SubstrateTransferStateControlBase
{
    // Default implementation - all logic uses base class no-op overrides
}
