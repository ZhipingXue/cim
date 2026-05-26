using Cim.ControlApp.Common;
using Cim.ControlApp.Common.Alarm;
using Cim.ControlApp.Common.ModuleLifecycle;
using Cim.ControlApp.Common.StateMachine;
using Microsoft.Extensions.Logging;

namespace Cim.SubstrateCacheControl.Services;

/// <summary>
/// Substrate cache module implementation with substrate transfer state management.
/// </summary>
public class SubCacheModule : ModuleStateBase
{
    private readonly ILogger<SubCacheModule>? _logger;

    public ISubstrateTransferStateControl TransferControl { get; }

    public SubCacheModule(
        ModuleInfo moduleInfo,
        IModuleAlarmCache alarmCache,
        ISubstrateTransferStateControl transferControl,
        ILogger<SubCacheModule>? logger = null)
        : base(moduleInfo, alarmCache)
    {
        _logger = logger;
        TransferControl = transferControl;
        TransferControl.StateChanged += OnTransferStateChanged;
    }

    protected override Task OnInitializingAsync(CancellationToken ct)
    {
        _logger?.LogInformation("SubstrateCache {ModuleName} ({ModuleId}) initializing...", ModuleInfo.DisplayName, ModuleInfo.Id);
        return Task.CompletedTask;
    }

    protected override Task OnRecoveringAsync(CancellationToken ct)
    {
        _logger?.LogInformation("SubstrateCache {ModuleName} ({ModuleId}) recovering...", ModuleInfo.DisplayName, ModuleInfo.Id);
        return Task.CompletedTask;
    }

    protected override Task OnShuttingDownAsync(CancellationToken ct)
    {
        _logger?.LogInformation("SubstrateCache {ModuleName} ({ModuleId}) shutting down...", ModuleInfo.DisplayName, ModuleInfo.Id);
        return Task.CompletedTask;
    }

    private void OnTransferStateChanged(object? sender, StateTransitionEventArgs<SubstrateTransferState, SubstrateTransferContext> args)
    {
        var ctx = args.Context;
        _logger?.LogInformation(
            "SubCache {ModuleName} transfer state: {From} -> {To}, Substrate={SubstrateId}, Source={Source}, Dest={Dest}",
            ModuleInfo.DisplayName,
            args.FromState,
            args.ToState,
            ctx?.SubstrateId,
            ctx?.SourceLocation,
            ctx?.DestinationLocation);
    }
}
