using Cim.ControlApp.Common;
using Cim.ControlApp.Common.Alarm;
using Cim.ControlApp.Common.ModuleLifecycle;
using Cim.ControlApp.Common.StateMachine;
using Microsoft.Extensions.Logging;

namespace Cim.ChamberControl.Services;

/// <summary>
/// Chamber module implementation with process and substrate transfer state management.
/// </summary>
public class ChamberModule : ModuleStateBase
{
    private readonly ILogger<ChamberModule>? _logger;

    public IProcessStateControl ProcessControl { get; }
    public ISubstrateTransferStateControl TransferControl { get; }

    public ChamberModule(
        ModuleInfo moduleInfo,
        IModuleAlarmCache alarmCache,
        IProcessStateControl processControl,
        ISubstrateTransferStateControl transferControl,
        ILogger<ChamberModule>? logger = null)
        : base(moduleInfo, alarmCache)
    {
        _logger = logger;
        ProcessControl = processControl;
        TransferControl = transferControl;

        // Wire up state change events
        ProcessControl.StateChanged += OnProcessStateChanged;
        TransferControl.StateChanged += OnTransferStateChanged;
    }

    protected override Task OnInitializingAsync(CancellationToken ct)
    {
        _logger?.LogInformation("Chamber {ModuleName} ({ModuleId}) initializing...", ModuleInfo.DisplayName, ModuleInfo.Id);
        return Task.CompletedTask;
    }

    protected override Task OnRecoveringAsync(CancellationToken ct)
    {
        _logger?.LogInformation("Chamber {ModuleName} ({ModuleId}) recovering...", ModuleInfo.DisplayName, ModuleInfo.Id);
        return Task.CompletedTask;
    }

    protected override Task OnShuttingDownAsync(CancellationToken ct)
    {
        _logger?.LogInformation("Chamber {ModuleName} ({ModuleId}) shutting down...", ModuleInfo.DisplayName, ModuleInfo.Id);
        return Task.CompletedTask;
    }

    private void OnProcessStateChanged(object? sender, StateTransitionEventArgs<ProcessState> args)
    {
        _logger?.LogInformation(
            "Chamber {ModuleName} process state: {From} -> {To}",
            ModuleInfo.DisplayName,
            args.FromState,
            args.ToState);
    }

    private void OnTransferStateChanged(object? sender, StateTransitionEventArgs<SubstrateTransferState, SubstrateTransferContext> args)
    {
        var ctx = args.Context;
        _logger?.LogInformation(
            "Chamber {ModuleName} transfer state: {From} -> {To}, Substrate={SubstrateId}, Source={Source}, Dest={Dest}",
            ModuleInfo.DisplayName,
            args.FromState,
            args.ToState,
            ctx?.SubstrateId,
            ctx?.SourceLocation,
            ctx?.DestinationLocation);
    }
}
