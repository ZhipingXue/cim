using Microsoft.Extensions.Configuration;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.Extensions.Logging;

namespace Cim.ControlApp.Common.DependencyInjection;

/// <summary>
/// Extension methods for registering CIM control app services with DI.
/// </summary>
public static class ServiceCollectionExtensions
{
    /// <summary>
    /// Uses chamber control app services (common + process state + substrate transfer + alarm).
    /// </summary>
    public static IServiceCollection UseChamber(this IServiceCollection services, IConfiguration configuration)
    {
        AddCommonServices(services, configuration);
        services.AddSingleton<StateMachine.IProcessStateControl, StateMachine.ProcessStateControl>();
        services.AddSingleton<StateMachine.ISubstrateTransferStateControl, StateMachine.SubstrateTransferStateControl>();
        AddAlarmReporter(services);
        return services;
    }

    /// <summary>
    /// Uses robot control app services (common + robot transfer + alarm).
    /// </summary>
    public static IServiceCollection UseRobot(this IServiceCollection services, IConfiguration configuration)
    {
        AddCommonServices(services, configuration);
        services.AddSingleton<StateMachine.IRobotTransferStateControl, StateMachine.RobotTransferStateControl>();
        AddAlarmReporter(services);
        return services;
    }

    /// <summary>
    /// Uses loadport control app services (common + alarm).
    /// </summary>
    public static IServiceCollection UseLoadPort(this IServiceCollection services, IConfiguration configuration)
    {
        AddCommonServices(services, configuration);
        AddAlarmReporter(services);
        return services;
    }

    /// <summary>
    /// Uses substrate cache control app services (common + substrate transfer + alarm).
    /// </summary>
    public static IServiceCollection UseSubstrateCache(this IServiceCollection services, IConfiguration configuration)
    {
        AddCommonServices(services, configuration);
        services.AddSingleton<StateMachine.ISubstrateTransferStateControl, StateMachine.SubstrateTransferStateControl>();
        AddAlarmReporter(services);
        return services;
    }

    private static void AddCommonServices(IServiceCollection services, IConfiguration configuration)
    {
        // ModuleInfo from configuration
        services.AddSingleton(sp => ModuleInfo.FromConfiguration(configuration));

        // Error handling
        services.AddSingleton<ErrorHandling.IErrorHandler>(sp =>
        {
            var logger = sp.GetService<ILogger<ErrorHandling.ErrorHandler>>();
            var alarmReporter = sp.GetService<Alarm.IAlarmReporter>();
            return new ErrorHandling.ErrorHandler(logger, alarmReporter);
        });

        // Data collection
        services.AddSingleton<DataCollection.IDataPublisher>(sp =>
        {
            var moduleInfo = sp.GetRequiredService<ModuleInfo>();
            var logger = sp.GetService<ILogger<DataCollection.DataPublisher>>();
            return new DataCollection.DataPublisher(moduleInfo.QualifiedId, logger);
        });

        // Service discovery
        services.AddSingleton<ServiceDiscovery.IServiceRegistryClient>(sp =>
        {
            var config = sp.GetRequiredService<IConfiguration>();
            var registryAddress = config["registry_address"] ?? "http://localhost:50000";
            var logger = sp.GetService<ILogger<ServiceDiscovery.ServiceRegistryClient>>();
            return new ServiceDiscovery.ServiceRegistryClient(registryAddress, logger);
        });
    }

    private static void AddAlarmReporter(IServiceCollection services)
    {
        services.AddSingleton<Alarm.IAlarmReporter>(sp =>
        {
            var moduleInfo = sp.GetRequiredService<ModuleInfo>();
            var logger = sp.GetService<ILogger<Alarm.AlarmReporter>>();
            return new Alarm.AlarmReporter(moduleInfo.EquipmentId, moduleInfo.QualifiedId, logger);
        });

        services.AddSingleton<Alarm.IModuleAlarmCache>(sp =>
        {
            var alarmReporter = sp.GetRequiredService<Alarm.IAlarmReporter>();
            var logger = sp.GetService<ILogger<Alarm.ModuleAlarmCache>>();
            return new Alarm.ModuleAlarmCache(alarmReporter, logger);
        });
    }
}
