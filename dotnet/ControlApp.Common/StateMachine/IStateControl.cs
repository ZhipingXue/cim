namespace Cim.ControlApp.Common.StateMachine;

/// <summary>
/// Generic state control interface with context support and transition results.
/// </summary>
public interface IStateControl<TState, TContext> where TState : struct, Enum
{
    TState CurrentState { get; }
    TContext? Context { get; }

    /// <summary>
    /// Checks if transition is allowed. Can be overridden for custom logic.
    /// </summary>
    bool CanTransitionTo(TState newState);

    /// <summary>
    /// Attempts state transition and returns result with typed error code and tips.
    /// </summary>
    Task<StateTransitionResult<TState, StateTransitionErrorCode>> TryTransitionToAsync(TState newState, TContext? context = default, CancellationToken ct = default);

    /// <summary>
    /// Forces state transition (throws on invalid transition).
    /// </summary>
    Task<TState> TransitionToAsync(TState newState, TContext? context = default, CancellationToken ct = default);

    event EventHandler<StateTransitionEventArgs<TState, TContext>>? StateChanged;
}

/// <summary>
/// Event args for state transitions with context.
/// </summary>
public class StateTransitionEventArgs<TState, TContext> : EventArgs where TState : struct, Enum
{
    public TState FromState { get; set; }
    public TState ToState { get; set; }
    public TContext? Context { get; set; }
    public DateTimeOffset Timestamp { get; set; }
    public string? Reason { get; set; }
}

/// <summary>
/// Legacy interface without context (for backward compatibility).
/// </summary>
public interface IStateControl<TState> where TState : struct, Enum
{
    TState CurrentState { get; }
    bool CanTransitionTo(TState newState);
    Task<StateTransitionResult<TState, StateTransitionErrorCode>> TryTransitionToAsync(TState newState, CancellationToken ct = default);
    Task<TState> TransitionToAsync(TState newState, CancellationToken ct = default);
    event EventHandler<StateTransitionEventArgs<TState>>? StateChanged;
}

/// <summary>
/// Legacy event args without context (for backward compatibility).
/// </summary>
public class StateTransitionEventArgs<TState> : EventArgs where TState : struct, Enum
{
    public TState FromState { get; set; }
    public TState ToState { get; set; }
    public DateTimeOffset Timestamp { get; set; }
    public string? Reason { get; set; }
}
