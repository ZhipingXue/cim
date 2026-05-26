using Cim.ControlApp.Common;
using Cim.ControlApp.Common.Alarm;
using Cim.ControlApp.Common.ModuleLifecycle;
using Microsoft.Extensions.Logging;

namespace Cim.LoadPortControl.Services;

/// <summary>
/// LoadPort module implementation.
/// </summary>
public class LoadPortModule : ModuleStateBase
{
    private readonly ILogger<LoadPortModule>? _logger;

    public LoadPortModule(ModuleInfo moduleInfo, IModuleAlarmCache alarmCache, ILogger<LoadPortModule>? logger = null)
        : base(moduleInfo, alarmCache)
    {
        _logger = logger;
    }

    protected override Task OnInitializingAsync(CancellationToken ct)
    {
        _logger?.LogInformation("LoadPort {ModuleName} ({ModuleId}) initializing...", ModuleInfo.DisplayName, ModuleInfo.Id);
        return Task.CompletedTask;
    }

    protected override Task OnRecoveringAsync(CancellationToken ct)
    {
        _logger?.LogInformation("LoadPort {ModuleName} ({ModuleId}) recovering...", ModuleInfo.DisplayName, ModuleInfo.Id);
        return Task.CompletedTask;
    }

    protected override Task OnShuttingDownAsync(CancellationToken ct)
    {
        _logger?.LogInformation("LoadPort {ModuleName} ({ModuleId}) shutting down...", ModuleInfo.DisplayName, ModuleInfo.Id);
        return Task.CompletedTask;
    }
}
