using Cim.ControlApp.Common;
using Cim.ControlApp.Common.Alarm;
using Cim.ControlApp.Common.DataCollection;
using Cim.ControlApp.Common.DependencyInjection;
using Cim.ControlApp.Common.ErrorHandling;
using Cim.ControlApp.Common.Grpc;
using Cim.ControlApp.Common.StateMachine;
using Cim.RobotControl.Services;
using Microsoft.AspNetCore.Builder;
using Microsoft.Extensions.Configuration;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.Extensions.Hosting;
using Microsoft.Extensions.Logging;

// Robot Control App Entry Point
// Uses DI/IoC with IConfiguration and ModuleInfo
// Exposes gRPC services for module state, robot transfer, and alarms

var builder = WebApplication.CreateBuilder(args);

// Configuration
builder.Configuration.AddJsonFile("config.json", optional: true, reloadOnChange: true);

// Logging
builder.Logging.AddConsole();
builder.Logging.SetMinimumLevel(LogLevel.Information);

// Register robot services (common + robot transfer + alarm)
builder.Services.UseRobot(builder.Configuration);

// Register robot-specific services with injected state control
builder.Services.AddSingleton<RobotModule>(sp =>
{
    var moduleInfo = sp.GetRequiredService<ModuleInfo>();
    var alarmCache = sp.GetRequiredService<IModuleAlarmCache>();
    var transferControl = sp.GetRequiredService<IRobotTransferStateControl>();
    var logger = sp.GetService<ILogger<RobotModule>>();
    return new RobotModule(moduleInfo, alarmCache, transferControl, logger);
});

// Register gRPC services
builder.Services.AddGrpc();
builder.Services.AddSingleton<ModuleStateGrpcService>(sp =>
{
    var moduleInfo = sp.GetRequiredService<ModuleInfo>();
    var robotModule = sp.GetRequiredService<RobotModule>();
    var logger = sp.GetRequiredService<ILogger<ModuleStateGrpcService>>();
    return new ModuleStateGrpcService(moduleInfo, robotModule, logger);
});
builder.Services.AddSingleton<RobotTransferGrpcService>(sp =>
{
    var moduleInfo = sp.GetRequiredService<ModuleInfo>();
    var transferControl = sp.GetRequiredService<IRobotTransferStateControl>();
    var logger = sp.GetRequiredService<ILogger<RobotTransferGrpcService>>();
    return new RobotTransferGrpcService(moduleInfo, transferControl, logger);
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
    var robotTransferService = sp.GetRequiredService<RobotTransferGrpcService>();
    var logger = sp.GetRequiredService<ILogger<ControlAppGrpcService>>();
    return new ControlAppGrpcService(moduleStateService, alarmService, logger, null, robotTransferService, null);
});

var app = builder.Build();

// Configure gRPC endpoints
app.MapGrpcService<ControlAppGrpcService>();

// Resolve services
var logger = app.Services.GetRequiredService<ILogger<Program>>();
var moduleInfo = app.Services.GetRequiredService<ModuleInfo>();
var robotControl = app.Services.GetRequiredService<IRobotTransferStateControl>();
var dataPublisher = app.Services.GetRequiredService<IDataPublisher>();
var errorHandler = app.Services.GetRequiredService<IErrorHandler>();
var robotModule = app.Services.GetRequiredService<RobotModule>();

logger.LogInformation("RobotControl starting with module {ModuleName} ({ModuleId})...", 
    moduleInfo.DisplayName, moduleInfo.Id);

// Wire up events
errorHandler.ErrorOccurred += (sender, args) =>
{
    logger.LogError("Error occurred: {ErrorCode} - {Description}", args.ErrorCode, args.Description);
};

robotModule.StateChanged += (sender, args) =>
{
    logger.LogInformation("Module state changed: {From} -> {To}", args.FromState, args.ToState);
};

// Initialize module
await robotModule.InitializeAsync();
logger.LogInformation("Robot {ModuleName} initialized. Module state: {ModuleState}, Transfer state: {TransferState}",
    moduleInfo.DisplayName, robotModule.CurrentState, robotControl.CurrentState);

// Publish status
await dataPublisher.PublishAsync("state", robotModule.CurrentState.ToString());

logger.LogInformation("RobotControl gRPC server running on {Url}. Press Ctrl+C to exit.", 
    builder.WebHost.GetSetting("urls") ?? "http://localhost:50101");
await app.RunAsync();
