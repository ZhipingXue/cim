using Microsoft.Extensions.Logging;

namespace Cim.ControlApp.Common.Alarm;

/// <summary>
/// Alarm severity levels.
/// </summary>
public enum AlarmSeverity
{
    Info = 0,
    Warning = 1,
    Error = 2,
    Critical = 3
}

/// <summary>
/// Alarm model.
/// </summary>
public class Alarm
{
    public int AlarmId { get; set; }
    public string AlarmCode { get; set; } = string.Empty;
    public string Description { get; set; } = string.Empty;
    public AlarmSeverity Severity { get; set; }
    public DateTimeOffset Timestamp { get; set; }
    public ModuleId Source { get; set; } = new();
}

/// <summary>
/// Module identifier.
/// </summary>
public class ModuleId
{
    public string EquipmentId { get; set; } = string.Empty;
    public string ModuleId_ { get; set; } = string.Empty;
    public string ModuleType { get; set; } = string.Empty;
}

/// <summary>
/// Alarm reporter interface.
/// </summary>
public interface IAlarmReporter
{
    Task ReportAlarmAsync(Alarm alarm, CancellationToken ct = default);
    Task ClearAlarmAsync(int alarmId, CancellationToken ct = default);
}

/// <summary>
/// Alarm reporter implementation that publishes alarms to E116 service via gRPC.
/// </summary>
public class AlarmReporter : IAlarmReporter
{
    private readonly string _moduleId;
    private readonly string _equipmentId;
    private readonly ILogger<AlarmReporter>? _logger;
    private int _nextAlarmId = 1;

    public AlarmReporter(string equipmentId, string moduleId, ILogger<AlarmReporter>? logger = null)
    {
        _equipmentId = equipmentId;
        _moduleId = moduleId;
        _logger = logger;
    }

    public Task ReportAlarmAsync(Alarm alarm, CancellationToken ct = default)
    {
        alarm.AlarmId = Interlocked.Increment(ref _nextAlarmId);
        alarm.Timestamp = DateTimeOffset.UtcNow;
        alarm.Source = new ModuleId
        {
            EquipmentId = _equipmentId,
            ModuleId_ = _moduleId,
            ModuleType = GetModuleType()
        };

        _logger?.LogWarning("Alarm {AlarmCode}: {Description}", alarm.AlarmCode, alarm.Description);

        // TODO: Send to E116 service via gRPC when available
        return Task.CompletedTask;
    }

    public Task ClearAlarmAsync(int alarmId, CancellationToken ct = default)
    {
        _logger?.LogInformation("Alarm {AlarmId} cleared", alarmId);

        // TODO: Send clear to E116 service via gRPC when available
        return Task.CompletedTask;
    }

    private string GetModuleType()
    {
        // Derive module type from module ID prefix or configuration
        return _moduleId.Split('-')[0] switch
        {
            "chamber" => "chamber",
            "robot" => "robot",
            "subcache" => "subcache",
            "loadport" => "loadport",
            _ => "unknown"
        };
    }
}
