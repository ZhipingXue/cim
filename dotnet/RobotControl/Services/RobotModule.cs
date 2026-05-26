using Cim.ControlApp.Common;
using Cim.ControlApp.Common.Alarm;
using Cim.ControlApp.Common.ModuleLifecycle;
using Cim.ControlApp.Common.StateMachine;
using Microsoft.Extensions.Logging;

namespace Cim.RobotControl.Services;

/// <summary>
/// Robot module implementation with transfer state management.
/// </summary>
public class RobotModule : ModuleStateBase
{
    private readonly ILogger<RobotModule>? _logger;

    public IRobotTransferStateControl TransferControl { get; }

    public RobotModule(
        ModuleInfo moduleInfo,
        IModuleAlarmCache alarmCache,
        IRobotTransferStateControl transferControl,
        ILogger<RobotModule>? logger = null)
        : base(moduleInfo, alarmCache)
    {
        _logger = logger;
        TransferControl = transferControl;
        TransferControl.StateChanged += OnTransferStateChanged;
    }

    protected override Task OnInitializingAsync(CancellationToken ct)
    {
        _logger?.LogInformation("Robot {ModuleName} ({ModuleId}) initializing...", ModuleInfo.DisplayName, ModuleInfo.Id);
        return Task.CompletedTask;
    }

    protected override Task OnRecoveringAsync(CancellationToken ct)
    {
        _logger?.LogInformation("Robot {ModuleName} ({ModuleId}) recovering...", ModuleInfo.DisplayName, ModuleInfo.Id);
        return Task.CompletedTask;
    }

    protected override Task OnShuttingDownAsync(CancellationToken ct)
    {
        _logger?.LogInformation("Robot {ModuleName} ({ModuleId}) shutting down...", ModuleInfo.DisplayName, ModuleInfo.Id);
        return Task.CompletedTask;
    }

    private void OnTransferStateChanged(object? sender, StateTransitionEventArgs<RobotTransferState> args)
    {
        _logger?.LogInformation(
            "Robot {ModuleName} transfer state: {From} -> {To}",
            ModuleInfo.DisplayName,
            args.FromState,
            args.ToState);
    }
}
