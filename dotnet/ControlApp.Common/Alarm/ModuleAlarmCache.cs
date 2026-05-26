using Microsoft.Extensions.Logging;

namespace Cim.ControlApp.Common.Alarm;

/// <summary>
/// Interface for module-local alarm cache that collects and manages active alarms.
/// </summary>
public interface IModuleAlarmCache
{
    /// <summary>
    /// All currently active (uncleared) alarms in this module.
    /// </summary>
    IReadOnlyCollection<Alarm> ActiveAlarms { get; }

    /// <summary>
    /// Number of active alarms.
    /// </summary>
    int ActiveAlarmCount { get; }

    /// <summary>
    /// Raised when a new alarm is reported.
    /// </summary>
    event EventHandler<AlarmEventArgs>? AlarmReported;

    /// <summary>
    /// Raised when an alarm is cleared.
    /// </summary>
    event EventHandler<AlarmEventArgs>? AlarmCleared;

    /// <summary>
    /// Reports a new alarm, adds it to the cache, and forwards to the global reporter.
    /// </summary>
    Task<Alarm> ReportAsync(string alarmCode, string description, AlarmSeverity severity = AlarmSeverity.Error, CancellationToken ct = default);

    /// <summary>
    /// Clears an alarm by ID, removes it from cache, and forwards to the global reporter.
    /// </summary>
    Task ClearAsync(int alarmId, CancellationToken ct = default);

    /// <summary>
    /// Clears all alarms matching the given alarm code.
    /// </summary>
    Task ClearByCodeAsync(string alarmCode, CancellationToken ct = default);

    /// <summary>
    /// Clears all active alarms.
    /// </summary>
    Task ClearAllAsync(CancellationToken ct = default);

    /// <summary>
    /// Gets all active alarms of a specific severity.
    /// </summary>
    IEnumerable<Alarm> GetAlarmsBySeverity(AlarmSeverity severity);

    /// <summary>
    /// Checks if any alarm with the given code is currently active.
    /// </summary>
    bool IsAlarmActive(string alarmCode);
}

/// <summary>
/// Event args for alarm events.
/// </summary>
public class AlarmEventArgs : EventArgs
{
    public Alarm Alarm { get; set; } = new();
    public DateTimeOffset Timestamp { get; set; }
}

/// <summary>
/// Module-local alarm cache that maintains active alarms and reports them via IAlarmReporter.
/// Thread-safe for concurrent access.
/// </summary>
public class ModuleAlarmCache : IModuleAlarmCache
{
    private readonly IAlarmReporter _alarmReporter;
    private readonly ILogger<ModuleAlarmCache>? _logger;
    private readonly Dictionary<int, Alarm> _activeAlarms = new();
    private readonly object _lock = new();
    private int _nextLocalAlarmId = 1;

    public IReadOnlyCollection<Alarm> ActiveAlarms
    {
        get
        {
            lock (_lock)
            {
                return _activeAlarms.Values.ToList().AsReadOnly();
            }
        }
    }

    public int ActiveAlarmCount
    {
        get
        {
            lock (_lock)
            {
                return _activeAlarms.Count;
            }
        }
    }

    public event EventHandler<AlarmEventArgs>? AlarmReported;
    public event EventHandler<AlarmEventArgs>? AlarmCleared;

    public ModuleAlarmCache(IAlarmReporter alarmReporter, ILogger<ModuleAlarmCache>? logger = null)
    {
        _alarmReporter = alarmReporter ?? throw new ArgumentNullException(nameof(alarmReporter));
        _logger = logger;
    }

    public async Task<Alarm> ReportAsync(string alarmCode, string description, AlarmSeverity severity = AlarmSeverity.Error, CancellationToken ct = default)
    {
        var alarm = new Alarm
        {
            AlarmId = Interlocked.Increment(ref _nextLocalAlarmId),
            AlarmCode = alarmCode,
            Description = description,
            Severity = severity,
            Timestamp = DateTimeOffset.UtcNow
        };

        lock (_lock)
        {
            _activeAlarms[alarm.AlarmId] = alarm;
        }

        _logger?.LogWarning("Alarm reported: {AlarmCode} - {Description} (Severity: {Severity})",
            alarmCode, description, severity);

        // Forward to global alarm reporter
        await _alarmReporter.ReportAlarmAsync(alarm, ct);

        AlarmReported?.Invoke(this, new AlarmEventArgs
        {
            Alarm = alarm,
            Timestamp = DateTimeOffset.UtcNow
        });

        return alarm;
    }

    public async Task ClearAsync(int alarmId, CancellationToken ct = default)
    {
        Alarm? alarm;
        lock (_lock)
        {
            if (!_activeAlarms.TryGetValue(alarmId, out alarm))
            {
                _logger?.LogWarning("Alarm {AlarmId} not found in cache, cannot clear", alarmId);
                return;
            }
            _activeAlarms.Remove(alarmId);
        }

        _logger?.LogInformation("Alarm cleared: {AlarmCode} - {Description}", alarm.AlarmCode, alarm.Description);

        // Forward to global alarm reporter
        await _alarmReporter.ClearAlarmAsync(alarmId, ct);

        AlarmCleared?.Invoke(this, new AlarmEventArgs
        {
            Alarm = alarm,
            Timestamp = DateTimeOffset.UtcNow
        });
    }

    public async Task ClearByCodeAsync(string alarmCode, CancellationToken ct = default)
    {
        int[] alarmIds;
        lock (_lock)
        {
            alarmIds = _activeAlarms.Values
                .Where(a => a.AlarmCode == alarmCode)
                .Select(a => a.AlarmId)
                .ToArray();
        }

        foreach (var id in alarmIds)
        {
            await ClearAsync(id, ct);
        }
    }

    public async Task ClearAllAsync(CancellationToken ct = default)
    {
        int[] alarmIds;
        lock (_lock)
        {
            alarmIds = _activeAlarms.Keys.ToArray();
            _activeAlarms.Clear();
        }

        foreach (var id in alarmIds)
        {
            await _alarmReporter.ClearAlarmAsync(id, ct);
        }

        _logger?.LogInformation("All alarms cleared ({Count} total)", alarmIds.Length);
    }

    public IEnumerable<Alarm> GetAlarmsBySeverity(AlarmSeverity severity)
    {
        lock (_lock)
        {
            return _activeAlarms.Values
                .Where(a => a.Severity == severity)
                .ToList();
        }
    }

    public bool IsAlarmActive(string alarmCode)
    {
        lock (_lock)
        {
            return _activeAlarms.Values.Any(a => a.AlarmCode == alarmCode);
        }
    }
}
