using Microsoft.Extensions.Configuration;
using Microsoft.Extensions.Hosting;
using Microsoft.Extensions.Logging;

namespace Cim.ControlApp.Common.Hosting;

using ServiceDiscovery;

/// <summary>
/// Base class for hosting a gRPC service with service registry integration.
/// </summary>
public abstract class GrpcServiceHost : IHostedService
{
    protected IConfiguration Configuration { get; }
    protected ILogger Logger { get; }
    protected IServiceRegistryClient RegistryClient { get; }

    protected GrpcServiceHost(
        IConfiguration configuration,
        ILogger logger,
        IServiceRegistryClient registryClient)
    {
        Configuration = configuration;
        Logger = logger;
        RegistryClient = registryClient;
    }

    public abstract Task StartAsync(CancellationToken cancellationToken);
    public abstract Task StopAsync(CancellationToken cancellationToken);

    protected abstract string ServiceId { get; }
    protected abstract string ServiceType { get; }
    protected abstract int Port { get; }
}
