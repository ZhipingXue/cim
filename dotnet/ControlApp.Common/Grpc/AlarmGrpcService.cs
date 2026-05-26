using Grpc.Core;
using Microsoft.Extensions.Logging;
using Cim.Common;
using Cim.Control;

namespace Cim.ControlApp.Common.Grpc;

/// <summary>
/// gRPC service for alarm management.
/// All control apps expose this service.
/// </summary>
public class AlarmGrpcService : GrpcServiceBase
{
    private readonly Alarm.IModuleAlarmCache _alarmCache;

    public AlarmGrpcService(
        ModuleInfo moduleInfo,
        Alarm.IModuleAlarmCache alarmCache,
        ILogger<AlarmGrpcService> logger)
        : base(moduleInfo, logger)
    {
        _alarmCache = alarmCache ?? throw new ArgumentNullException(nameof(alarmCache));
    }

    /// <summary>
    /// Reports a new alarm.
    /// </summary>
    public async Task<ReportAlarmResponse> ReportAlarmAsync(ReportAlarmRequest request, ServerCallContext context)
    {
        var alarm = await _alarmCache.ReportAsync(
            request.Alarm.AlarmCode,
            request.Alarm.Description,
            (Alarm.AlarmSeverity)(int)request.Alarm.Severity,
            context.CancellationToken);

        return new ReportAlarmResponse 
        { 
            Status = Ok($"Alarm {alarm.AlarmId} reported") 
        };
    }

    /// <summary>
    /// Clears an alarm by ID.
    /// Note: Uses ReportAlarmResponse as the proto doesn't have a dedicated ClearAlarmResponse.
    /// </summary>
    public async Task<ReportAlarmResponse> ClearAlarmAsync(int alarmId, ServerCallContext context)
    {
        await _alarmCache.ClearAsync(alarmId, context.CancellationToken);

        return new ReportAlarmResponse 
        { 
            Status = Ok($"Alarm {alarmId} cleared") 
        };
    }

    /// <summary>
    /// Gets all active alarms.
    /// Note: Uses ReportAlarmResponse as the proto doesn't have a dedicated GetActiveAlarmsResponse.
    /// </summary>
    public Task<ReportAlarmResponse> GetActiveAlarmsAsync(ServerCallContext context)
    {
        var alarms = _alarmCache.ActiveAlarms.Select(a => new global::Cim.Common.Alarm
        {
            AlarmId = a.AlarmId,
            AlarmCode = a.AlarmCode,
            Description = a.Description,
            Severity = (AlarmSeverity)(int)a.Severity,
            Timestamp = Google.Protobuf.WellKnownTypes.Timestamp.FromDateTime(a.Timestamp.DateTime),
            Source = new global::Cim.Common.ModuleId 
            { 
                EquipmentId = ModuleInfo.EquipmentId, 
                ModuleId_ = ModuleInfo.Name,
                ModuleType = ModuleInfo.ModuleType
            }
        }).ToList();

        // Note: The proto doesn't have a dedicated GetActiveAlarmsResponse, 
        // so we return a simple success response
        return Task.FromResult(new ReportAlarmResponse
        {
            Status = Ok($"{alarms.Count} active alarms")
        });
    }

    /// <summary>
    /// Clears all active alarms.
    /// </summary>
    public async Task<ReportAlarmResponse> ClearAllAlarmsAsync(ServerCallContext context)
    {
        await _alarmCache.ClearAllAsync(context.CancellationToken);

        return new ReportAlarmResponse 
        { 
            Status = Ok("All alarms cleared") 
        };
    }
}
