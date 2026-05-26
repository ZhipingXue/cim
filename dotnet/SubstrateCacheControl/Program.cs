using Cim.ControlApp.Common;
using Cim.ControlApp.Common.Alarm;
using Cim.ControlApp.Common.DataCollection;
using Cim.ControlApp.Common.DependencyInjection;
using Cim.ControlApp.Common.ErrorHandling;
using Cim.ControlApp.Common.Grpc;
using Cim.ControlApp.Common.StateMachine;
using Cim.SubstrateCacheControl.Services;
using Microsoft.AspNetCore.Builder;
using Microsoft.Extensions.Configuration;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.Extensions.Hosting;
using Microsoft.Extensions.Logging;

// SubstrateCache Control App Entry Point
// Uses DI/IoC with IConfiguration and ModuleInfo
// Exposes gRPC services for module state, substrate transfer, and alarms

var builder = WebApplication.CreateBuilder(args);

// Configuration
builder.Configuration.AddJsonFile("config.json", optional: true, reloadOnChange: true);

// Logging
builder.Logging.AddConsole();
builder.Logging.SetMinimumLevel(LogLevel.Information);

// Register substrate cache services (common + substrate transfer + alarm)
builder.Services.UseSubstrateCache(builder.Configuration);

// Register substrate cache-specific services with injected state control
builder.Services.AddSingleton<SubCacheModule>(sp =>
{
    var moduleInfo = sp.GetRequiredService<ModuleInfo>();
    var alarmCache = sp.GetRequiredService<IModuleAlarmCache>();
    var transferControl = sp.GetRequiredService<ISubstrateTransferStateControl>();
    var logger = sp.GetService<ILogger<SubCacheModule>>();
    return new SubCacheModule(moduleInfo, alarmCache, transferControl, logger);
});

// Register gRPC services
builder.Services.AddGrpc();
builder.Services.AddSingleton<ModuleStateGrpcService>(sp =>
{
    var moduleInfo = sp.GetRequiredService<ModuleInfo>();
    var subCacheModule = sp.GetRequiredService<SubCacheModule>();
    var logger = sp.GetRequiredService<ILogger<ModuleStateGrpcService>>();
    return new ModuleStateGrpcService(moduleInfo, subCacheModule, logger);
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
    var substrateTransferService = sp.GetRequiredService<SubstrateTransferGrpcService>();
    var logger = sp.GetRequiredService<ILogger<ControlAppGrpcService>>();
    return new ControlAppGrpcService(moduleStateService, alarmService, logger, null, null, substrateTransferService);
});

var app = builder.Build();

// Configure gRPC endpoints
app.MapGrpcService<ControlAppGrpcService>();

// Resolve services
var logger = app.Services.GetRequiredService<ILogger<Program>>();
var moduleInfo = app.Services.GetRequiredService<ModuleInfo>();
var dataPublisher = app.Services.GetRequiredService<IDataPublisher>();
var errorHandler = app.Services.GetRequiredService<IErrorHandler>();
var subCacheModule = app.Services.GetRequiredService<SubCacheModule>();

logger.LogInformation("SubstrateCacheControl starting with module {ModuleName} ({ModuleId})...", 
    moduleInfo.DisplayName, moduleInfo.Id);

// Wire up events
errorHandler.ErrorOccurred += (sender, args) =>
{
    logger.LogError("Error occurred: {ErrorCode} - {Description}", args.ErrorCode, args.Description);
};

subCacheModule.StateChanged += (sender, args) =>
{
    logger.LogInformation("Module state changed: {From} -> {To}", args.FromState, args.ToState);
};

// Initialize module
await subCacheModule.InitializeAsync();
logger.LogInformation("SubstrateCache {ModuleName} initialized. Current state: {State}", 
    moduleInfo.DisplayName, subCacheModule.CurrentState);

await subCacheModule.GoOnlineAsync();
logger.LogInformation("SubstrateCache {ModuleName} online. Current state: {State}", 
    moduleInfo.DisplayName, subCacheModule.CurrentState);

// Publish status
await dataPublisher.PublishAsync("state", subCacheModule.CurrentState.ToString());

logger.LogInformation("SubstrateCacheControl gRPC server running on {Url}. Press Ctrl+C to exit.", 
    builder.WebHost.GetSetting("urls") ?? "http://localhost:50103");
await app.RunAsync();
