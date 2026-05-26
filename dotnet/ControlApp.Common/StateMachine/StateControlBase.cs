namespace Cim.ControlApp.Common.StateMachine;

/// <summary>
/// Base implementation of generic state control with transition validation, context support, and result-based transitions.
/// Thread-safe using lock.
/// </summary>
public abstract class StateControlBase<TState, TContext> : IStateControl<TState, TContext> where TState : struct, Enum
{
    private readonly Dictionary<TState, HashSet<TState>> _transitions;
    private readonly object _lock = new();
    private TState _currentState;
    private TContext? _context;

    public TState CurrentState
    {
        get
        {
            lock (_lock)
            {
                return _currentState;
            }
        }
    }

    public TContext? Context
    {
        get
        {
            lock (_lock)
            {
                return _context;
            }
        }
    }

    public event EventHandler<StateTransitionEventArgs<TState, TContext>>? StateChanged;

    protected StateControlBase(TState initialState, Dictionary<TState, HashSet<TState>> transitions)
    {
        _currentState = initialState;
        _transitions = transitions;
    }

    /// <summary>
    /// Checks if transition is allowed. Override for custom transition logic.
    /// Base implementation checks the transition graph.
    /// </summary>
    public virtual bool CanTransitionTo(TState newState)
    {
        lock (_lock)
        {
            if (!_transitions.TryGetValue(_currentState, out var allowed))
                return false;
            return allowed.Contains(newState);
        }
    }

    /// <summary>
    /// Attempts state transition and returns detailed result with error code and tips.
    /// </summary>
    public Task<StateTransitionResult<TState, StateTransitionErrorCode>> TryTransitionToAsync(TState newState, TContext? context = default, CancellationToken ct = default)
    {
        lock (_lock)
        {
            var fromState = _currentState;

            if (!CanTransitionTo(newState))
            {
                var errorMsg = $"Invalid transition: {fromState} -> {newState}";
                var tips = new Dictionary<string, string>
                {
                    ["current_state"] = $"Current state is {fromState}",
                    ["requested_state"] = $"Requested transition to {newState}",
                    ["allowed_transitions"] = $"Allowed transitions from {fromState}: {GetAllowedTransitionsString(fromState)}",
                    ["suggestion"] = $"Ensure the module is in a state that allows transition to {newState}"
                };
                var result = StateTransitionResult<TState, StateTransitionErrorCode>.Failure(
                    StateTransitionErrorCode.InvalidTransition, fromState, newState, errorMsg, tips);
                return Task.FromResult(result);
            }

            _currentState = newState;
            _context = context;

            var args = new StateTransitionEventArgs<TState, TContext>
            {
                FromState = fromState,
                ToState = newState,
                Context = context,
                Timestamp = DateTimeOffset.UtcNow
            };

            OnStateChanged(args);
            StateChanged?.Invoke(this, args);

            var successTips = new Dictionary<string, string>
            {
                ["from_state"] = $"Previous state: {fromState}",
                ["to_state"] = $"New state: {newState}",
                ["timestamp"] = DateTimeOffset.UtcNow.ToString("O")
            };
            var successResult = StateTransitionResult<TState, StateTransitionErrorCode>.Success(fromState, newState, successTips);
            return Task.FromResult(successResult);
        }
    }

    /// <summary>
    /// Forces state transition. Throws InvalidOperationException on invalid transition.
    /// </summary>
    public Task<TState> TransitionToAsync(TState newState, TContext? context = default, CancellationToken ct = default)
    {
        lock (_lock)
        {
            if (!CanTransitionTo(newState))
            {
                throw new InvalidOperationException(
                    $"Invalid transition: {_currentState} -> {newState}. " +
                    $"Allowed transitions from {_currentState}: {GetAllowedTransitionsString(_currentState)}");
            }

            var fromState = _currentState;
            _currentState = newState;
            _context = context;

            var args = new StateTransitionEventArgs<TState, TContext>
            {
                FromState = fromState,
                ToState = newState,
                Context = context,
                Timestamp = DateTimeOffset.UtcNow
            };

            OnStateChanged(args);
            StateChanged?.Invoke(this, args);

            return Task.FromResult(newState);
        }
    }

    protected virtual void OnStateChanged(StateTransitionEventArgs<TState, TContext> args) { }

    private string GetAllowedTransitionsString(TState fromState)
    {
        if (_transitions.TryGetValue(fromState, out var allowed))
            return string.Join(", ", allowed);
        return "none";
    }
}

/// <summary>
/// Legacy base class without context (for backward compatibility).
/// </summary>
public abstract class StateControlBase<TState> : IStateControl<TState> where TState : struct, Enum
{
    private readonly Dictionary<TState, HashSet<TState>> _transitions;
    private readonly object _lock = new();
    private TState _currentState;

    public TState CurrentState
    {
        get
        {
            lock (_lock)
            {
                return _currentState;
            }
        }
    }

    public event EventHandler<StateTransitionEventArgs<TState>>? StateChanged;

    protected StateControlBase(TState initialState, Dictionary<TState, HashSet<TState>> transitions)
    {
        _currentState = initialState;
        _transitions = transitions;
    }

    /// <summary>
    /// Checks if transition is allowed. Override for custom transition logic.
    /// </summary>
    public virtual bool CanTransitionTo(TState newState)
    {
        lock (_lock)
        {
            if (!_transitions.TryGetValue(_currentState, out var allowed))
                return false;
            return allowed.Contains(newState);
        }
    }

    /// <summary>
    /// Attempts state transition and returns detailed result with error code and tips.
    /// </summary>
    public Task<StateTransitionResult<TState, StateTransitionErrorCode>> TryTransitionToAsync(TState newState, CancellationToken ct = default)
    {
        lock (_lock)
        {
            var fromState = _currentState;

            if (!CanTransitionTo(newState))
            {
                var errorMsg = $"Invalid transition: {fromState} -> {newState}";
                var tips = new Dictionary<string, string>
                {
                    ["current_state"] = $"Current state is {fromState}",
                    ["requested_state"] = $"Requested transition to {newState}",
                    ["allowed_transitions"] = $"Allowed transitions from {fromState}: {GetAllowedTransitionsString(fromState)}",
                    ["suggestion"] = $"Ensure the module is in a state that allows transition to {newState}"
                };
                var result = StateTransitionResult<TState, StateTransitionErrorCode>.Failure(
                    StateTransitionErrorCode.InvalidTransition, fromState, newState, errorMsg, tips);
                return Task.FromResult(result);
            }

            _currentState = newState;

            var args = new StateTransitionEventArgs<TState>
            {
                FromState = fromState,
                ToState = newState,
                Timestamp = DateTimeOffset.UtcNow
            };

            OnStateChanged(args);
            StateChanged?.Invoke(this, args);

            var successTips = new Dictionary<string, string>
            {
                ["from_state"] = $"Previous state: {fromState}",
                ["to_state"] = $"New state: {newState}",
                ["timestamp"] = DateTimeOffset.UtcNow.ToString("O")
            };
            var successResult = StateTransitionResult<TState, StateTransitionErrorCode>.Success(fromState, newState, successTips);
            return Task.FromResult(successResult);
        }
    }

    /// <summary>
    /// Forces state transition. Throws InvalidOperationException on invalid transition.
    /// </summary>
    public Task<TState> TransitionToAsync(TState newState, CancellationToken ct = default)
    {
        lock (_lock)
        {
            if (!CanTransitionTo(newState))
            {
                throw new InvalidOperationException(
                    $"Invalid transition: {_currentState} -> {newState}. " +
                    $"Allowed transitions from {_currentState}: {GetAllowedTransitionsString(_currentState)}");
            }

            var fromState = _currentState;
            _currentState = newState;

            var args = new StateTransitionEventArgs<TState>
            {
                FromState = fromState,
                ToState = newState,
                Timestamp = DateTimeOffset.UtcNow
            };

            OnStateChanged(args);
            StateChanged?.Invoke(this, args);

            return Task.FromResult(newState);
        }
    }

    protected virtual void OnStateChanged(StateTransitionEventArgs<TState> args) { }

    private string GetAllowedTransitionsString(TState fromState)
    {
        if (_transitions.TryGetValue(fromState, out var allowed))
            return string.Join(", ", allowed);
        return "none";
    }
}
