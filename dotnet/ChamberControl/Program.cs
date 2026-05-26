using Cim.ChamberControl.Services;
using Cim.ControlApp.Common;
using Cim.ControlApp.Common.Alarm;
using Cim.ControlApp.Common.DataCollection;
using Cim.ControlApp.Common.DependencyInjection;
using Cim.ControlApp.Common.ErrorHandling;
using Cim.ControlApp.Common.Grpc;
using Cim.ControlApp.Common.StateMachine;
using Microsoft.AspNetCore.Builder;
using Microsoft.AspNetCore.Hosting;
using Microsoft.Extensions.Configuration;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.Extensions.Hosting;
using Microsoft.Extensions.Logging;

// Chamber Control App Entry Point
// Uses DI/IoC with IConfiguration and ModuleInfo
// Exposes gRPC services for module state, process control, substrate transfer, and alarms

var builder = WebApplication.CreateBuilder(args);

// Configuration
builder.Configuration.AddJsonFile("config.json", optional: true, reloadOnChange: true);

// Logging
builder.Logging.AddConsole();
builder.Logging.SetMinimumLevel(LogLevel.Information);

// Register chamber services (common + process state + substrate transfer + alarm)
builder.Services.UseChamber(builder.Configuration);

// Register chamber-specific services with injected state controls
builder.Services.AddSingleton<ChamberModule>(sp =>
{
    var moduleInfo = sp.GetRequiredService<ModuleInfo>();
    var alarmCache = sp.GetRequiredService<IModuleAlarmCache>();
    var processControl = sp.GetRequiredService<IProcessStateControl>();
    var transferControl = sp.GetRequiredService<ISubstrateTransferStateControl>();
    var logger = sp.GetService<ILogger<ChamberModule>>();
    return new ChamberModule(moduleInfo, alarmCache, processControl, transferControl, logger);
});

// Register gRPC services
builder.Services.AddGrpc();
builder.Services.AddSingleton<ModuleStateGrpcService>(sp =>
{
    var moduleInfo = sp.GetRequiredService<ModuleInfo>();
    var chamberModule = sp.GetRequiredService<ChamberModule>();
    var logger = sp.GetRequiredService<ILogger<ModuleStateGrpcService>>();
    return new ModuleStateGrpcService(moduleInfo, chamberModule, logger);
});
builder.Services.AddSingleton<ProcessControlGrpcService>(sp =>
{
    var moduleInfo = sp.GetRequiredService<ModuleInfo>();
    var processControl = sp.GetRequiredService<IProcessStateControl>();
    var logger = sp.GetRequiredService<ILogger<ProcessControlGrpcService>>();
    return new ProcessControlGrpcService(moduleInfo, processControl, logger);
});
builder.Services.AddSingleton<SubstrateTransferGrpcService>(sp =>
{
    var moduleInfo = sp.GetRequiredService<ModuleInfo>();
    var transferControl = sp.GetRequiredService<ISubstrateTransferStateControl>();
    var logger = sp.GetRequiredService<ILogger<SubstrateTransferGrpcService>>();
    return new SubstrateTransferGrpcService(moduleInfo, transferControl, logger);
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
    var processControlService = sp.GetRequiredService<ProcessControlGrpcService>();
    var substrateTransferService = sp.GetRequiredService<SubstrateTransferGrpcService>();
    var logger = sp.GetRequiredService<ILogger<ControlAppGrpcService>>();
    return new ControlAppGrpcService(moduleStateService, alarmService, logger, processControlService, null, substrateTransferService);
});

var app = builder.Build();

// Configure gRPC endpoints
app.MapGrpcService<ControlAppGrpcService>();

// Resolve services
var logger = app.Services.GetRequiredService<ILogger<Program>>();
var moduleInfo = app.Services.GetRequiredService<ModuleInfo>();
var processControl = app.Services.GetRequiredService<IProcessStateControl>();
var dataPublisher = app.Services.GetRequiredService<IDataPublisher>();
var errorHandler = app.Services.GetRequiredService<IErrorHandler>();
var chamberModule = app.Services.GetRequiredService<ChamberModule>();

logger.LogInformation("ChamberControl starting with module {ModuleName} ({ModuleId})...", 
    moduleInfo.DisplayName, moduleInfo.Id);

// Wire up events
errorHandler.ErrorOccurred += (sender, args) =>
{
    logger.LogError("Error occurred: {ErrorCode} - {Description}", args.ErrorCode, args.Description);
};

chamberModule.StateChanged += (sender, args) =>
{
    logger.LogInformation("Module state changed: {From} -> {To}", args.FromState, args.ToState);
};

// Initialize module
await chamberModule.InitializeAsync();
logger.LogInformation("Chamber {ModuleName} initialized. Module state: {ModuleState}, Process state: {ProcessState}",
    moduleInfo.DisplayName, chamberModule.CurrentState, processControl.CurrentState);

// Publish status
await dataPublisher.PublishAsync("state", chamberModule.CurrentState.ToString());

logger.LogInformation("ChamberControl gRPC server running on {Url}. Press Ctrl+C to exit.", 
    builder.WebHost.GetSetting("urls") ?? "http://localhost:50100");
await app.RunAsync();
