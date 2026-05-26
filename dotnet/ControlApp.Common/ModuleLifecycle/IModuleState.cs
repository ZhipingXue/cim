namespace Cim.ControlApp.Common.ModuleLifecycle;

/// <summary>
/// Generic module lifecycle state.
/// </summary>
public enum ModuleState
{
    Unspecified = 0,
    Offline = 1,
    Initializing = 2,
    Online = 3,
    Processing = 4,
    Error = 5
}

/// <summary>
/// Module lifecycle state interface.
/// </summary>
public interface IModuleState
{
    ModuleState CurrentState { get; }
    Task InitializeAsync(CancellationToken ct = default);
    Task GoOnlineAsync(CancellationToken ct = default);
    Task GoOfflineAsync(CancellationToken ct = default);
    Task HandleErrorAsync(string errorCode, string description, CancellationToken ct = default);
    event EventHandler<ModuleStateChangedEventArgs>? StateChanged;
}

/// <summary>
/// Event args for module state changes.
/// </summary>
public class ModuleStateChangedEventArgs : EventArgs
{
    public ModuleState FromState { get; set; }
    public ModuleState ToState { get; set; }
    public DateTimeOffset Timestamp { get; set; }
}
