namespace Cim.ControlApp.Common.StateMachine;

/// <summary>
/// Default error codes for state transitions.
/// </summary>
public enum StateTransitionErrorCode : int
{
    Success = 0,
    InvalidTransition = 1,
    PreConditionNotMet = 2,
    PostConditionFailed = 3,
    Timeout = 4,
    Cancelled = 5,
    InternalError = 99
}

/// <summary>
/// Result of a state transition attempt with typed error code and diagnostic tips.
/// </summary>
public readonly struct StateTransitionResult<TState, TErrorCode>
    where TState : struct, Enum
    where TErrorCode : struct, Enum
{
    public TErrorCode ErrorCode { get; }
    public bool IsSuccess => Convert.ToInt32(ErrorCode) == 0;
    public TState? FromState { get; }
    public TState ToState { get; }
    public string? ErrorMessage { get; }

    /// <summary>
    /// Diagnostic tips for troubleshooting the transition result.
    /// Key: tip category/identifier, Value: detailed message.
    /// </summary>
    public IReadOnlyDictionary<string, string> Tips { get; }

    private StateTransitionResult(TErrorCode errorCode, TState? fromState, TState toState, string? errorMessage, Dictionary<string, string>? tips)
    {
        ErrorCode = errorCode;
        FromState = fromState;
        ToState = toState;
        ErrorMessage = errorMessage;
        Tips = tips ?? new Dictionary<string, string>();
    }

    public static StateTransitionResult<TState, TErrorCode> Success(TState fromState, TState toState, Dictionary<string, string>? tips = null)
    {
        var successCode = (TErrorCode)Enum.ToObject(typeof(TErrorCode), 0);
        return new StateTransitionResult<TState, TErrorCode>(successCode, fromState, toState, null, tips);
    }

    public static StateTransitionResult<TState, TErrorCode> Failure(TErrorCode errorCode, TState fromState, TState toState, string errorMessage, Dictionary<string, string>? tips = null)
        => new(errorCode, fromState, toState, errorMessage, tips);

    public static StateTransitionResult<TState, TErrorCode> Failure(TErrorCode errorCode, TState toState, string errorMessage, Dictionary<string, string>? tips = null)
        => new(errorCode, null, toState, errorMessage, tips);

    public override string ToString()
        => IsSuccess
            ? $"Success: {FromState} -> {ToState}"
            : $"Failure [{ErrorCode}]: {FromState} -> {ToState}, Error: {ErrorMessage}";
}

/// <summary>
/// Convenience type alias using default StateTransitionErrorCode.
/// </summary>
public readonly struct StateTransitionResult<TState> where TState : struct, Enum
{
    private readonly StateTransitionResult<TState, StateTransitionErrorCode> _inner;

    public StateTransitionErrorCode ErrorCode => _inner.ErrorCode;
    public bool IsSuccess => _inner.IsSuccess;
    public TState? FromState => _inner.FromState;
    public TState ToState => _inner.ToState;
    public string? ErrorMessage => _inner.ErrorMessage;
    public IReadOnlyDictionary<string, string> Tips => _inner.Tips;

    private StateTransitionResult(StateTransitionResult<TState, StateTransitionErrorCode> inner)
    {
        _inner = inner;
    }

    public static StateTransitionResult<TState> Success(TState fromState, TState toState, Dictionary<string, string>? tips = null)
        => new(StateTransitionResult<TState, StateTransitionErrorCode>.Success(fromState, toState, tips));

    public static StateTransitionResult<TState> Failure(StateTransitionErrorCode errorCode, TState fromState, TState toState, string errorMessage, Dictionary<string, string>? tips = null)
        => new(StateTransitionResult<TState, StateTransitionErrorCode>.Failure(errorCode, fromState, toState, errorMessage, tips));

    public static StateTransitionResult<TState> Failure(StateTransitionErrorCode errorCode, TState toState, string errorMessage, Dictionary<string, string>? tips = null)
        => new(StateTransitionResult<TState, StateTransitionErrorCode>.Failure(errorCode, toState, errorMessage, tips));

    public override string ToString() => _inner.ToString();
}
