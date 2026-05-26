using Microsoft.Extensions.Hosting;

namespace Cim.ControlApp.Common.Hosting;

/// <summary>
/// Windows Service host wrapper for control apps.
/// </summary>
public class WindowsServiceHost<T> : BackgroundService where T : class, IHostedService
{
    private readonly T _hostedService;

    public WindowsServiceHost(T hostedService)
    {
        _hostedService = hostedService;
    }

    protected override async Task ExecuteAsync(CancellationToken stoppingToken)
    {
        await _hostedService.StartAsync(stoppingToken);
        await Task.Delay(Timeout.Infinite, stoppingToken);
        await _hostedService.StopAsync(CancellationToken.None);
    }
}
