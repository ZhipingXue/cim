namespace Cim.ControlApp.Common.ModuleLifecycle;

/// <summary>
/// Base implementation of module lifecycle state management.
/// All control apps (chamber, robot, subcache, loadport) inherit from this.
/// </summary>
public abstract class ModuleStateBase : IModuleState
{
    private readonly object _lock = new();
    private ModuleState _currentState = ModuleState.Offline;

    public ModuleInfo ModuleInfo { get; }
    public Alarm.IModuleAlarmCache AlarmCache { get; }

    public ModuleState CurrentState
    {
        get
        {
            lock (_lock)
            {
                return _currentState;
            }
        }
    }

    public event EventHandler<ModuleStateChangedEventArgs>? StateChanged;

    protected ModuleStateBase(ModuleInfo moduleInfo, Alarm.IModuleAlarmCache alarmCache)
    {
        ModuleInfo = moduleInfo ?? throw new ArgumentNullException(nameof(moduleInfo));
        AlarmCache = alarmCache ?? throw new ArgumentNullException(nameof(alarmCache));
    }

    public async Task InitializeAsync(CancellationToken ct = default)
    {
        await TransitionToAsync(ModuleState.Initializing, ct);
        await OnInitializingAsync(ct);
        await TransitionToAsync(ModuleState.Online, ct);
    }

    public async Task GoOnlineAsync(CancellationToken ct = default)
    {
        if (CurrentState == ModuleState.Offline)
        {
            await InitializeAsync(ct);
        }
        else if (CurrentState == ModuleState.Error)
        {
            await TransitionToAsync(ModuleState.Initializing, ct);
            await OnRecoveringAsync(ct);
            await TransitionToAsync(ModuleState.Online, ct);
        }
    }

    public async Task GoOfflineAsync(CancellationToken ct = default)
    {
        await OnShuttingDownAsync(ct);
        await TransitionToAsync(ModuleState.Offline, ct);
    }

    public async Task HandleErrorAsync(string errorCode, string description, CancellationToken ct = default)
    {
        await OnErrorAsync(errorCode, description, ct);
        await TransitionToAsync(ModuleState.Error, ct);
    }

    protected async Task TransitionToAsync(ModuleState newState, CancellationToken ct)
    {
        lock (_lock)
        {
            var fromState = _currentState;
            _currentState = newState;

            var args = new ModuleStateChangedEventArgs
            {
                FromState = fromState,
                ToState = newState,
                Timestamp = DateTimeOffset.UtcNow
            };

            OnStateChanged(args);
            StateChanged?.Invoke(this, args);
        }

        await Task.CompletedTask;
    }

    protected virtual void OnStateChanged(ModuleStateChangedEventArgs args) { }

    // Override points for derived classes
    protected virtual Task OnInitializingAsync(CancellationToken ct) => Task.CompletedTask;
    protected virtual Task OnRecoveringAsync(CancellationToken ct) => Task.CompletedTask;
    protected virtual Task OnShuttingDownAsync(CancellationToken ct) => Task.CompletedTask;
    protected virtual Task OnErrorAsync(string errorCode, string description, CancellationToken ct) => Task.CompletedTask;
}
