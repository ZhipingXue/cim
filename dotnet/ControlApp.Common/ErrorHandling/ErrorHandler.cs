using Microsoft.Extensions.Logging;

namespace Cim.ControlApp.Common.ErrorHandling;

/// <summary>
/// Error event arguments.
/// </summary>
public class ErrorEventArgs : EventArgs
{
    public string ErrorCode { get; set; } = string.Empty;
    public string Description { get; set; } = string.Empty;
    public string Context { get; set; } = string.Empty;
    public DateTimeOffset Timestamp { get; set; }
    public Exception? Exception { get; set; }
}

/// <summary>
/// Error handler interface.
/// </summary>
public interface IErrorHandler
{
    event EventHandler<ErrorEventArgs>? ErrorOccurred;
    Task HandleErrorAsync(Exception exception, string context, CancellationToken ct = default);
    Task HandleErrorAsync(string errorCode, string description, string context, CancellationToken ct = default);
}

/// <summary>
/// Centralized error handler that logs errors and reports alarms.
/// </summary>
public class ErrorHandler : IErrorHandler
{
    private readonly ILogger<ErrorHandler>? _logger;
    private readonly Alarm.IAlarmReporter? _alarmReporter;

    public event EventHandler<ErrorEventArgs>? ErrorOccurred;

    public ErrorHandler(ILogger<ErrorHandler>? logger = null, Alarm.IAlarmReporter? alarmReporter = null)
    {
        _logger = logger;
        _alarmReporter = alarmReporter;
    }

    public async Task HandleErrorAsync(Exception exception, string context, CancellationToken ct = default)
    {
        var args = new ErrorEventArgs
        {
            ErrorCode = exception.GetType().Name,
            Description = exception.Message,
            Context = context,
            Timestamp = DateTimeOffset.UtcNow,
            Exception = exception
        };

        _logger?.LogError(exception, "Error in {Context}: {Message}", context, exception.Message);
        ErrorOccurred?.Invoke(this, args);

        if (_alarmReporter != null)
        {
            await _alarmReporter.ReportAlarmAsync(new Alarm.Alarm
            {
                AlarmCode = args.ErrorCode,
                Description = args.Description,
                Severity = Alarm.AlarmSeverity.Error
            }, ct);
        }
    }

    public async Task HandleErrorAsync(string errorCode, string description, string context, CancellationToken ct = default)
    {
        var args = new ErrorEventArgs
        {
            ErrorCode = errorCode,
            Description = description,
            Context = context,
            Timestamp = DateTimeOffset.UtcNow
        };

        _logger?.LogError("Error {ErrorCode} in {Context}: {Description}", errorCode, context, description);
        ErrorOccurred?.Invoke(this, args);

        if (_alarmReporter != null)
        {
            await _alarmReporter.ReportAlarmAsync(new Alarm.Alarm
            {
                AlarmCode = errorCode,
                Description = description,
                Severity = Alarm.AlarmSeverity.Error
            }, ct);
        }
    }
}
