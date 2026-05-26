namespace Cim.ControlApp.Common.StateMachine;

/// <summary>
/// Simple result for action operations (On***Async methods).
/// Less complex than StateTransitionResult for internal operations.
/// </summary>
public readonly struct ActionResult
{
    public bool IsSuccess { get; }
    public string? ErrorMessage { get; }
    public Dictionary<string, string> Details { get; }

    private ActionResult(bool isSuccess, string? errorMessage, Dictionary<string, string>? details)
    {
        IsSuccess = isSuccess;
        ErrorMessage = errorMessage;
        Details = details ?? new Dictionary<string, string>();
    }

    public static ActionResult Success(Dictionary<string, string>? details = null)
        => new(true, null, details);

    public static ActionResult Success(string detailKey, string detailValue)
        => new(true, null, new Dictionary<string, string> { [detailKey] = detailValue });

    public static ActionResult Failure(string errorMessage, Dictionary<string, string>? details = null)
        => new(false, errorMessage, details);

    public static ActionResult Failure(string errorMessage, string detailKey, string detailValue)
        => new(false, errorMessage, new Dictionary<string, string> { [detailKey] = detailValue });

    public override string ToString()
        => IsSuccess ? "Success" : $"Failure: {ErrorMessage}";
}
