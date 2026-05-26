using Cim.ControlApp.Common;
using Cim.ControlApp.Common.Alarm;
using Cim.ControlApp.Common.DataCollection;
using Cim.ControlApp.Common.DependencyInjection;
using Cim.ControlApp.Common.ErrorHandling;
using Cim.ControlApp.Common.Grpc;
using Cim.LoadPortControl.Services;
using Microsoft.AspNetCore.Builder;
using Microsoft.Extensions.Configuration;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.Extensions.Hosting;
using Microsoft.Extensions.Logging;

// LoadPort Control App Entry Point
// Uses DI/IoC with IConfiguration and ModuleInfo
// Exposes gRPC services for module state and alarms

var builder = WebApplication.CreateBuilder(args);

// Configuration
builder.Configuration.AddJsonFile("config.json", optional: true, reloadOnChange: true);

// Logging
builder.Logging.AddConsole();
builder.Logging.SetMinimumLevel(LogLevel.Information);

// Register loadport services (common + alarm)
builder.Services.UseLoadPort(builder.Configuration);

// Register loadport-specific services
builder.Services.AddSingleton<LoadPortModule>(sp =>
{
    var moduleInfo = sp.GetRequiredService<ModuleInfo>();
    var alarmCache = sp.GetRequiredService<IModuleAlarmCache>();
    var logger = sp.GetService<ILogger<LoadPortModule>>();
    return new LoadPortModule(moduleInfo, alarmCache, logger);
});

// Register gRPC services
builder.Services.AddGrpc();
builder.Services.AddSingleton<ModuleStateGrpcService>(sp =>
{
    var moduleInfo = sp.GetRequiredService<ModuleInfo>();
    var loadPortModule = sp.GetRequiredService<LoadPortModule>();
    var logger = sp.GetRequiredService<ILogger<ModuleStateGrpcService>>();
    return new ModuleStateGrpcService(moduleInfo, loadPortModule, logger);
});
builder.Services.AddSingleton<AlarmGrpcService>(sp =>
{
    var moduleInfo = sp.GetRequiredService<ModuleInfo>();
    var alarmCache = sp.GetRequiredService<IModuleAlarmCache>();
    var logger = sp.GetRequiredService<ILogger<AlarmGrpcService>>();
    return new AlarmGrpcService(moduleInfo, alarmCache, logger);
});
builder.Services.AddSingleton<ControlAppGrpcService>(sp =>
{
    var moduleStateService = sp.GetRequiredService<ModuleStateGrpcService>();
    var alarmService = sp.GetRequiredService<AlarmGrpcService>();
    var logger = sp.GetRequiredService<ILogger<ControlAppGrpcService>>();
    return new ControlAppGrpcService(moduleStateService, alarmService, logger);
});

var app = builder.Build();

// Configure gRPC endpoints
app.MapGrpcService<ControlAppGrpcService>();

// Resolve services
var logger = app.Services.GetRequiredService<ILogger<Program>>();
var moduleInfo = app.Services.GetRequiredService<ModuleInfo>();
var dataPublisher = app.Services.GetRequiredService<IDataPublisher>();
var errorHandler = app.Services.GetRequiredService<IErrorHandler>();
var loadPortModule = app.Services.GetRequiredService<LoadPortModule>();

logger.LogInformation("LoadPortControl starting with module {ModuleName} ({ModuleId})...", 
    moduleInfo.DisplayName, moduleInfo.Id);

// Wire up events
errorHandler.ErrorOccurred += (sender, args) =>
{
    logger.LogError("Error occurred: {ErrorCode} - {Description}", args.ErrorCode, args.Description);
};

loadPortModule.StateChanged += (sender, args) =>
{
    logger.LogInformation("Module state changed: {From} -> {To}", args.FromState, args.ToState);
};

// Initialize module
await loadPortModule.InitializeAsync();
logger.LogInformation("LoadPort {ModuleName} initialized. Current state: {State}", 
    moduleInfo.DisplayName, loadPortModule.CurrentState);

await loadPortModule.GoOnlineAsync();
logger.LogInformation("LoadPort {ModuleName} online. Current state: {State}", 
    moduleInfo.DisplayName, loadPortModule.CurrentState);

// Publish status
await dataPublisher.PublishAsync("state", loadPortModule.CurrentState.ToString());

logger.LogInformation("LoadPortControl gRPC server running on {Url}. Press Ctrl+C to exit.", 
    builder.WebHost.GetSetting("urls") ?? "http://localhost:50102");
await app.RunAsync();
